use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const SCHEMA_VERSION: &str = "0.1.0";
const PROJECTS_DIR_NAME: &str = "Asset Forge Projects";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectFile {
    schema_version: String,
    project_id: String,
    name: String,
    #[serde(default)]
    github_repository_url: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    notes: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSummary {
    project_id: String,
    name: String,
    path: String,
    github_repository_url: String,
    local_folder: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenProject {
    project: ProjectSummary,
    project_json: ProjectFile,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectDetailsUpdate {
    path: String,
    description: String,
    notes: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectLinksUpdate {
    path: String,
    github_repository_url: String,
    local_folder: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSyncRequest {
    path: String,
    action: SyncAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum SyncAction {
    Pull,
    Push,
    Status,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSyncResult {
    command: String,
    output: String,
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn now_isoish() -> String {
    format!("{}Z", now_millis())
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();

    for character in value.to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
        } else if (character.is_ascii_whitespace() || character == '-' || character == '_')
            && !slug.ends_with('-')
        {
            slug.push('-');
        }
    }

    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "untitled-project".to_string()
    } else {
        trimmed
    }
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "Could not resolve a home directory for project storage.".to_string())
}

fn projects_root() -> Result<PathBuf, String> {
    Ok(home_dir()?.join("Documents").join(PROJECTS_DIR_NAME))
}

fn app_data_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .unwrap_or(home_dir()?.join(".config"));
    Ok(base.join("Asset Forge"))
}

fn machine_links_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("machine-project-links.json"))
}

fn read_machine_links() -> Result<HashMap<String, String>, String> {
    let path = machine_links_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))
}

fn write_machine_links(links: &HashMap<String, String>) -> Result<(), String> {
    let path = machine_links_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }

    let serialized = serde_json::to_string_pretty(links)
        .map_err(|error| format!("Could not serialize machine project links: {error}"))?;
    fs::write(&path, format!("{serialized}\n"))
        .map_err(|error| format!("Could not write {}: {error}", path.display()))
}

fn ensure_project_subdirs(project_path: &Path) -> Result<(), String> {
    for directory in [
        "data",
        "assets/images",
        "assets/backgrounds",
        "assets/tokens",
        "assets/generated",
        "icons",
        "templates",
        "style-kits",
        "builds",
        "exports",
    ] {
        fs::create_dir_all(project_path.join(directory))
            .map_err(|error| format!("Could not create project folder {directory}: {error}"))?;
    }

    Ok(())
}

fn read_project(project_path: &Path) -> Result<ProjectFile, String> {
    let project_json_path = project_path.join("project.json");
    let contents = fs::read_to_string(&project_json_path).map_err(|error| {
        format!(
            "Could not read {}: {error}",
            project_json_path.to_string_lossy()
        )
    })?;

    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "Could not parse {}: {error}",
            project_json_path.to_string_lossy()
        )
    })
}

fn project_summary(
    project_path: &Path,
    project: &ProjectFile,
    machine_links: &HashMap<String, String>,
) -> ProjectSummary {
    ProjectSummary {
        project_id: project.project_id.clone(),
        name: project.name.clone(),
        path: project_path.to_string_lossy().to_string(),
        github_repository_url: project.github_repository_url.clone(),
        local_folder: machine_links
            .get(&project.project_id)
            .cloned()
            .unwrap_or_default(),
        created_at: project.created_at.clone(),
        updated_at: project.updated_at.clone(),
    }
}

fn write_project(project_path: &Path, project: &ProjectFile) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(project)
        .map_err(|error| format!("Could not serialize project metadata: {error}"))?;
    let project_json_path = project_path.join("project.json");
    fs::write(&project_json_path, format!("{serialized}\n")).map_err(|error| {
        format!(
            "Could not write {}: {error}",
            project_json_path.to_string_lossy()
        )
    })
}

#[tauri::command]
fn list_projects() -> Result<Vec<ProjectSummary>, String> {
    let root = projects_root()?;
    let machine_links = read_machine_links()?;
    fs::create_dir_all(&root)
        .map_err(|error| format!("Could not create projects root {}: {error}", root.display()))?;

    let mut projects = Vec::new();

    for entry in fs::read_dir(&root)
        .map_err(|error| format!("Could not read projects root {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| format!("Could not read project entry: {error}"))?;
        let project_path = entry.path();

        if !project_path.is_dir() || !project_path.join("project.json").exists() {
            continue;
        }

        match read_project(&project_path) {
            Ok(project) => projects.push(project_summary(&project_path, &project, &machine_links)),
            Err(error) => eprintln!("{error}"),
        }
    }

    projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(projects)
}

#[tauri::command]
fn create_project(name: String) -> Result<OpenProject, String> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err("Project name is required.".to_string());
    }

    let root = projects_root()?;
    fs::create_dir_all(&root)
        .map_err(|error| format!("Could not create projects root {}: {error}", root.display()))?;

    let timestamp = now_millis();
    let project_id = format!(
        "proj_{}_{}",
        slugify(trimmed_name).replace('-', "_"),
        timestamp
    );
    let project_path = root.join(format!("{}-{}", slugify(trimmed_name), timestamp));

    fs::create_dir_all(&project_path).map_err(|error| {
        format!(
            "Could not create project folder {}: {error}",
            project_path.display()
        )
    })?;
    ensure_project_subdirs(&project_path)?;

    let now = now_isoish();
    let project = ProjectFile {
        schema_version: SCHEMA_VERSION.to_string(),
        project_id,
        name: trimmed_name.to_string(),
        github_repository_url: String::new(),
        description: String::new(),
        notes: String::new(),
        created_at: now.clone(),
        updated_at: now,
    };

    write_project(&project_path, &project)?;

    let machine_links = read_machine_links()?;
    Ok(OpenProject {
        project: project_summary(&project_path, &project, &machine_links),
        project_json: project,
    })
}

#[tauri::command]
fn open_project(path: String) -> Result<OpenProject, String> {
    let project_path = PathBuf::from(path);
    let project = read_project(&project_path)?;
    let machine_links = read_machine_links()?;

    Ok(OpenProject {
        project: project_summary(&project_path, &project, &machine_links),
        project_json: project,
    })
}

#[tauri::command]
fn update_project_details(update: ProjectDetailsUpdate) -> Result<OpenProject, String> {
    let project_path = PathBuf::from(update.path);
    let mut project = read_project(&project_path)?;
    project.description = update.description;
    project.notes = update.notes;
    project.updated_at = now_isoish();

    write_project(&project_path, &project)?;
    let machine_links = read_machine_links()?;

    Ok(OpenProject {
        project: project_summary(&project_path, &project, &machine_links),
        project_json: project,
    })
}

#[tauri::command]
fn update_project_links(update: ProjectLinksUpdate) -> Result<OpenProject, String> {
    let project_path = PathBuf::from(update.path);
    let mut project = read_project(&project_path)?;
    project.github_repository_url = update.github_repository_url.trim().to_string();
    project.updated_at = now_isoish();
    write_project(&project_path, &project)?;

    let mut machine_links = read_machine_links()?;
    let local_folder = update.local_folder.trim();
    if local_folder.is_empty() {
        machine_links.remove(&project.project_id);
    } else {
        machine_links.insert(project.project_id.clone(), local_folder.to_string());
    }
    write_machine_links(&machine_links)?;

    Ok(OpenProject {
        project: project_summary(&project_path, &project, &machine_links),
        project_json: project,
    })
}

fn run_git_command(local_folder: &Path, args: &[&str]) -> Result<ProjectSyncResult, String> {
    if !local_folder.exists() {
        return Err(format!(
            "Local folder does not exist on this machine: {}",
            local_folder.display()
        ));
    }

    let output = Command::new("git")
        .args(args)
        .current_dir(local_folder)
        .output()
        .map_err(|error| format!("Could not run git: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined_output = [stdout.trim(), stderr.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if !output.status.success() {
        return Err(format!(
            "git {} failed:\n{}",
            args.join(" "),
            combined_output
        ));
    }

    Ok(ProjectSyncResult {
        command: format!("git {}", args.join(" ")),
        output: if combined_output.is_empty() {
            "Command completed without output.".to_string()
        } else {
            combined_output
        },
    })
}

#[tauri::command]
fn sync_external_project(request: ProjectSyncRequest) -> Result<ProjectSyncResult, String> {
    let project_path = PathBuf::from(request.path);
    let project = read_project(&project_path)?;
    let machine_links = read_machine_links()?;
    let local_folder = machine_links
        .get(&project.project_id)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Set a local folder for this machine before syncing.".to_string())?;
    let local_folder_path = PathBuf::from(local_folder);

    if !local_folder_path.join(".git").exists() {
        return Err(format!(
            "Local folder is not a Git repository: {}",
            local_folder_path.display()
        ));
    }

    if !project.github_repository_url.trim().is_empty() {
        let _ = run_git_command(
            &local_folder_path,
            &[
                "remote",
                "set-url",
                "origin",
                &project.github_repository_url,
            ],
        )
        .or_else(|_| {
            run_git_command(
                &local_folder_path,
                &["remote", "add", "origin", &project.github_repository_url],
            )
        })?;
    }

    match request.action {
        SyncAction::Pull => run_git_command(&local_folder_path, &["pull", "--ff-only"]),
        SyncAction::Push => run_git_command(&local_folder_path, &["push"]),
        SyncAction::Status => {
            run_git_command(&local_folder_path, &["status", "--short", "--branch"])
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            list_projects,
            create_project,
            open_project,
            update_project_details,
            update_project_links,
            sync_external_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running Asset Forge");
}
