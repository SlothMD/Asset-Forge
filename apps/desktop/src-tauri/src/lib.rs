use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::BufWriter,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ShipModelTransform {
    #[serde(default)]
    yaw_degrees: f32,
    #[serde(default)]
    pitch_degrees: f32,
    #[serde(default)]
    roll_degrees: f32,
    #[serde(default = "default_transform_scale")]
    scale: f32,
}

impl Default for ShipModelTransform {
    fn default() -> Self {
        Self {
            yaw_degrees: 0.0,
            pitch_degrees: 0.0,
            roll_degrees: 0.0,
            scale: default_transform_scale(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShipModelOrientationManifest {
    schema_version: String,
    updated_at: String,
    models: HashMap<String, ShipModelTransform>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShipModelOrientationEntry {
    model_path: String,
    file_name: String,
    absolute_path: String,
    transform: ShipModelTransform,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShipModelOrientationCatalog {
    manifest_path: String,
    models: Vec<ShipModelOrientationEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShipModelOrientationUpdate {
    project_path: String,
    model_path: String,
    transform: ShipModelTransform,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShipModelPreviewRequest {
    project_path: String,
    model_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageScanRequest {
    project_path: String,
    source_folder: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageAssetEntry {
    absolute_path: String,
    relative_path: String,
    file_name: String,
    extension: String,
    inferred_type: String,
    width: u32,
    height: u32,
    file_size_bytes: u64,
    opportunities: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageScanResult {
    source_folder: String,
    assets: Vec<ImageAssetEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageOptimizeRequest {
    project_path: String,
    source_folder: String,
    staging_folder: String,
    target_max_dimension: u32,
    output_format: String,
    jpeg_quality: u8,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageOptimizeEntry {
    source_path: String,
    output_path: String,
    source_size_bytes: u64,
    output_size_bytes: u64,
    source_width: u32,
    source_height: u32,
    output_width: u32,
    output_height: u32,
    action: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageOptimizeResult {
    staging_folder: String,
    outputs: Vec<ImageOptimizeEntry>,
}

fn default_transform_scale() -> f32 {
    1.0
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn local_folder_for_project(project_path: &Path) -> Result<PathBuf, String> {
    let project = read_project(project_path)?;
    let machine_links = read_machine_links()?;
    let local_folder = machine_links
        .get(&project.project_id)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Set a local folder for this machine first.".to_string())?;
    let local_folder_path = PathBuf::from(local_folder);
    if !local_folder_path.exists() {
        return Err(format!(
            "Local folder does not exist on this machine: {}",
            local_folder_path.display()
        ));
    }

    Ok(local_folder_path)
}

fn ship_model_manifest_path(local_folder: &Path) -> PathBuf {
    local_folder
        .join("content")
        .join("assets")
        .join("ship-model-orientation.manifest.json")
}

fn read_ship_model_manifest(path: &Path) -> Result<ShipModelOrientationManifest, String> {
    if !path.exists() {
        return Ok(ShipModelOrientationManifest {
            schema_version: SCHEMA_VERSION.to_string(),
            updated_at: now_isoish(),
            models: HashMap::new(),
        });
    }

    let contents = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))
}

fn write_ship_model_manifest(
    path: &Path,
    manifest: &ShipModelOrientationManifest,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }

    let serialized = serde_json::to_string_pretty(manifest)
        .map_err(|error| format!("Could not serialize ship model orientation manifest: {error}"))?;
    fs::write(path, format!("{serialized}\n"))
        .map_err(|error| format!("Could not write {}: {error}", path.display()))
}

fn collect_glb_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(root)
        .map_err(|error| format!("Could not read model directory {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| format!("Could not read model directory entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_glb_files(&path, files)?;
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("glb"))
        {
            files.push(path);
        }
    }

    Ok(())
}

fn collect_image_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(root)
        .map_err(|error| format!("Could not read {}: {error}", root.display()))?
    {
        let entry =
            entry.map_err(|error| format!("Could not read entry in {}: {error}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_image_files(&path, files)?;
        } else if is_supported_image_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn is_supported_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg"
            )
        })
        .unwrap_or(false)
}

fn resolve_project_folder(project_path: &str, folder: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(folder.trim());
    if path.is_absolute() {
        return Ok(path);
    }

    Ok(local_folder_for_project(&PathBuf::from(project_path))?.join(path))
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn infer_image_type(path: &Path) -> String {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if name.contains("normal") || name.contains("_n.") || name.contains("-n.") {
        "normal".to_string()
    } else if name.contains("rough") {
        "roughness".to_string()
    } else if name.contains("metal") {
        "metallic".to_string()
    } else if name.contains("ao") || name.contains("ambient") || name.contains("occlusion") {
        "ambient-occlusion".to_string()
    } else if name.contains("height") || name.contains("displace") {
        "height".to_string()
    } else if name.contains("albedo")
        || name.contains("base_color")
        || name.contains("basecolor")
        || name.contains("color")
    {
        "albedo".to_string()
    } else if name.contains("icon") || name.contains("ui") {
        "ui-icon".to_string()
    } else if name.contains("screenshot") || name.contains("capsule") || name.contains("preview") {
        "marketing".to_string()
    } else {
        "unknown".to_string()
    }
}

fn image_opportunities(
    path: &Path,
    image_type: &str,
    width: u32,
    height: u32,
    size: u64,
) -> Vec<String> {
    let mut opportunities = Vec::new();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if width.max(height) > 2048 {
        opportunities.push("downscale-to-2k-runtime".to_string());
    }
    if !width.is_power_of_two() || !height.is_power_of_two() {
        opportunities.push("non-power-of-two-dimensions".to_string());
    }
    if extension == "png" && size > 2_500_000 && image_type != "normal" && image_type != "ui-icon"
    {
        opportunities.push("png-large-consider-jpeg".to_string());
    }
    if image_type == "unknown" {
        opportunities.push("type-needs-review".to_string());
    }

    opportunities
}

fn res_path_for_model(local_folder: &Path, model_path: &Path) -> String {
    let relative = model_path.strip_prefix(local_folder).unwrap_or(model_path);
    format!(
        "res://{}",
        relative
            .to_string_lossy()
            .replace('\\', "/")
            .trim_start_matches('/')
    )
}

fn model_path_from_res_path(local_folder: &Path, model_path: &str) -> Result<PathBuf, String> {
    let relative = model_path
        .strip_prefix("res://")
        .ok_or_else(|| format!("Unsupported model path: {}", model_path))?;
    let candidate = local_folder.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    let model_root = local_folder.join("assets").join("models").join("ships");
    let canonical_model_root = model_root.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve model folder {}: {}",
            model_root.display(),
            error
        )
    })?;
    let canonical_candidate = candidate.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve model path {}: {}",
            candidate.display(),
            error
        )
    })?;

    if !canonical_candidate.starts_with(&canonical_model_root) {
        return Err(format!(
            "Model path is outside the ship model folder: {}",
            model_path
        ));
    }

    Ok(canonical_candidate)
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

#[tauri::command]
fn list_ship_model_orientations(project_path: String) -> Result<ShipModelOrientationCatalog, String> {
    let project_path = PathBuf::from(project_path);
    let local_folder = local_folder_for_project(&project_path)?;
    let manifest_path = ship_model_manifest_path(&local_folder);
    let manifest = read_ship_model_manifest(&manifest_path)?;
    let mut files = Vec::new();
    collect_glb_files(&local_folder.join("assets").join("models").join("ships"), &mut files)?;
    files.sort();

    let models = files
        .into_iter()
        .map(|path| {
            let model_path = res_path_for_model(&local_folder, &path);
            let transform = manifest
                .models
                .get(&model_path)
                .cloned()
                .unwrap_or_default();
            ShipModelOrientationEntry {
                file_name: path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| model_path.clone()),
                model_path,
                absolute_path: path.to_string_lossy().to_string(),
                transform,
            }
        })
        .collect();

    Ok(ShipModelOrientationCatalog {
        manifest_path: manifest_path.to_string_lossy().to_string(),
        models,
    })
}

#[tauri::command]
fn save_ship_model_orientation(
    update: ShipModelOrientationUpdate,
) -> Result<ShipModelOrientationCatalog, String> {
    let project_path = PathBuf::from(&update.project_path);
    let local_folder = local_folder_for_project(&project_path)?;
    let manifest_path = ship_model_manifest_path(&local_folder);
    let mut manifest = read_ship_model_manifest(&manifest_path)?;
    manifest.updated_at = now_isoish();
    manifest
        .models
        .insert(update.model_path, update.transform);
    write_ship_model_manifest(&manifest_path, &manifest)?;
    list_ship_model_orientations(update.project_path)
}

#[tauri::command]
fn load_ship_model_preview(request: ShipModelPreviewRequest) -> Result<Vec<u8>, String> {
    let project_path = PathBuf::from(request.project_path);
    let local_folder = local_folder_for_project(&project_path)?;
    let model_path = model_path_from_res_path(&local_folder, &request.model_path)?;
    fs::read(&model_path).map_err(|error| {
        format!(
            "Unable to read model preview {}: {}",
            model_path.display(),
            error
        )
    })
}

#[tauri::command]
fn scan_image_assets(request: ImageScanRequest) -> Result<ImageScanResult, String> {
    let source_folder = resolve_project_folder(&request.project_path, &request.source_folder)?;
    if !source_folder.exists() {
        return Err(format!(
            "Image source folder does not exist: {}",
            source_folder.display()
        ));
    }

    let mut files = Vec::new();
    collect_image_files(&source_folder, &mut files)?;
    files.sort();

    let mut assets = Vec::new();
    for path in files {
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("Could not read metadata for {}: {error}", path.display()))?;
        let (width, height) = image::image_dimensions(&path)
            .map_err(|error| format!("Could not read image {}: {error}", path.display()))?;
        let inferred_type = infer_image_type(&path);
        let opportunities =
            image_opportunities(&path, &inferred_type, width, height, metadata.len());
        assets.push(ImageAssetEntry {
            absolute_path: path.to_string_lossy().to_string(),
            relative_path: relative_path_string(&source_folder, &path),
            file_name: path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default(),
            extension: path
                .extension()
                .map(|extension| extension.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default(),
            inferred_type,
            width,
            height,
            file_size_bytes: metadata.len(),
            opportunities,
        });
    }

    Ok(ImageScanResult {
        source_folder: source_folder.to_string_lossy().to_string(),
        assets,
    })
}

#[tauri::command]
fn optimize_image_assets(request: ImageOptimizeRequest) -> Result<ImageOptimizeResult, String> {
    let source_folder = resolve_project_folder(&request.project_path, &request.source_folder)?;
    let staging_folder = resolve_project_folder(&request.project_path, &request.staging_folder)?;
    let target_max = request.target_max_dimension.clamp(256, 8192);
    let output_format = request.output_format.trim().to_ascii_lowercase();
    let jpeg_quality = request.jpeg_quality.clamp(35, 100);
    if !matches!(output_format.as_str(), "jpg" | "jpeg" | "png") {
        return Err("Output format must be png or jpg.".to_string());
    }

    let mut files = Vec::new();
    collect_image_files(&source_folder, &mut files)?;
    files.sort();
    fs::create_dir_all(&staging_folder).map_err(|error| {
        format!(
            "Could not create staging folder {}: {error}",
            staging_folder.display()
        )
    })?;

    let mut outputs = Vec::new();
    for path in files {
        let relative = path.strip_prefix(&source_folder).unwrap_or(&path);
        let mut output_path = staging_folder.join(relative);
        output_path.set_extension(if output_format == "jpeg" { "jpg" } else { &output_format });
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!("Could not create output folder {}: {error}", parent.display())
            })?;
        }

        let source_metadata = fs::metadata(&path)
            .map_err(|error| format!("Could not read metadata for {}: {error}", path.display()))?;
        let image = image::ImageReader::open(&path)
            .map_err(|error| format!("Could not open image {}: {error}", path.display()))?
            .decode()
            .map_err(|error| format!("Could not decode image {}: {error}", path.display()))?;
        let source_width = image.width();
        let source_height = image.height();
        let max_dimension = source_width.max(source_height);
        let optimized = if max_dimension > target_max {
            let ratio = target_max as f32 / max_dimension as f32;
            let width = ((source_width as f32 * ratio).round() as u32).max(1);
            let height = ((source_height as f32 * ratio).round() as u32).max(1);
            image.resize(width, height, image::imageops::FilterType::Lanczos3)
        } else {
            image
        };

        if output_format == "png" {
            optimized
                .save_with_format(&output_path, image::ImageFormat::Png)
                .map_err(|error| format!("Could not write {}: {error}", output_path.display()))?;
        } else {
            let output = fs::File::create(&output_path)
                .map_err(|error| format!("Could not create {}: {error}", output_path.display()))?;
            let mut writer = BufWriter::new(output);
            let rgb = optimized.to_rgb8();
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, jpeg_quality);
            encoder
                .encode_image(&rgb)
                .map_err(|error| format!("Could not write {}: {error}", output_path.display()))?;
        }

        let output_metadata = fs::metadata(&output_path)
            .map_err(|error| format!("Could not read {}: {error}", output_path.display()))?;
        outputs.push(ImageOptimizeEntry {
            source_path: path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            source_size_bytes: source_metadata.len(),
            output_size_bytes: output_metadata.len(),
            source_width,
            source_height,
            output_width: optimized.width(),
            output_height: optimized.height(),
            action: if max_dimension > target_max {
                format!("resized-to-{}", target_max)
            } else {
                "reencoded-copy".to_string()
            },
        });
    }

    Ok(ImageOptimizeResult {
        staging_folder: staging_folder.to_string_lossy().to_string(),
        outputs,
    })
}

#[tauri::command]
fn close_app(app: tauri::AppHandle) {
    app.exit(0);
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
            sync_external_project,
            list_ship_model_orientations,
            save_ship_model_orientation,
            load_ship_model_preview,
            scan_image_assets,
            optimize_image_assets,
            close_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running Asset Forge");
}
