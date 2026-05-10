use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
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

fn project_summary(project_path: &Path, project: &ProjectFile) -> ProjectSummary {
    ProjectSummary {
        project_id: project.project_id.clone(),
        name: project.name.clone(),
        path: project_path.to_string_lossy().to_string(),
        created_at: project.created_at.clone(),
        updated_at: project.updated_at.clone(),
    }
}

#[tauri::command]
fn list_projects() -> Result<Vec<ProjectSummary>, String> {
    let root = projects_root()?;
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
            Ok(project) => projects.push(project_summary(&project_path, &project)),
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
    let project_id = format!("proj_{}_{}", slugify(trimmed_name).replace('-', "_"), timestamp);
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
        description: String::new(),
        notes: String::new(),
        created_at: now.clone(),
        updated_at: now,
    };

    let project_json_path = project_path.join("project.json");
    let serialized = serde_json::to_string_pretty(&project)
        .map_err(|error| format!("Could not serialize project metadata: {error}"))?;
    fs::write(&project_json_path, format!("{serialized}\n")).map_err(|error| {
        format!(
            "Could not write {}: {error}",
            project_json_path.to_string_lossy()
        )
    })?;

    Ok(OpenProject {
        project: project_summary(&project_path, &project),
        project_json: project,
    })
}

#[tauri::command]
fn open_project(path: String) -> Result<OpenProject, String> {
    let project_path = PathBuf::from(path);
    let project = read_project(&project_path)?;

    Ok(OpenProject {
        project: project_summary(&project_path, &project),
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

    let serialized = serde_json::to_string_pretty(&project)
        .map_err(|error| format!("Could not serialize project metadata: {error}"))?;
    let project_json_path = project_path.join("project.json");
    fs::write(&project_json_path, format!("{serialized}\n")).map_err(|error| {
        format!(
            "Could not write {}: {error}",
            project_json_path.to_string_lossy()
        )
    })?;

    Ok(OpenProject {
        project: project_summary(&project_path, &project),
        project_json: project,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            list_projects,
            create_project,
            open_project,
            update_project_details
        ])
        .run(tauri::generate_context!())
        .expect("error while running Asset Forge");
}
