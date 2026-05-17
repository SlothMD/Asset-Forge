use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
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
    #[serde(default)]
    audit: Vec<HandoffAuditEntry>,
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
    model_folder: String,
    models: Vec<ShipModelOrientationEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShipModelOrientationRequest {
    project_path: String,
    model_folder: String,
    manifest_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShipModelOrientationUpdate {
    project_path: String,
    model_folder: String,
    manifest_path: String,
    model_path: String,
    transform: ShipModelTransform,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShipModelPreviewRequest {
    project_path: String,
    model_folder: String,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelScanRequest {
    project_path: String,
    source_folder: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelAssetEntry {
    absolute_path: String,
    relative_path: String,
    file_name: String,
    file_size_bytes: u64,
    triangle_count: u64,
    opportunities: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelScanResult {
    source_folder: String,
    assets: Vec<ModelAssetEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelOptimizeRequest {
    project_path: String,
    source_folder: String,
    staging_folder: String,
    manifest_path: String,
    compression: String,
    texture_compress: String,
    simplify: bool,
    target_triangles: u64,
    force: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelOptimizeEntry {
    source_path: String,
    output_path: String,
    source_size_bytes: u64,
    output_size_bytes: u64,
    source_triangles: u64,
    target_triangles: u64,
    simplify_ratio: f32,
    action: String,
    command: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelOptimizeResult {
    staging_folder: String,
    manifest_path: String,
    outputs: Vec<ModelOptimizeEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelOptimizationManifest {
    schema_version: String,
    updated_at: String,
    tool: String,
    source_folder: String,
    staging_folder: String,
    outputs: Vec<ModelOptimizeEntry>,
    audit: Vec<HandoffAuditEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrellisBatchRequest {
    project_path: String,
    source_folder: String,
    final_output_folder: String,
    workflow_path: String,
    comfy_url: String,
    comfy_output_folder: String,
    staging_folder: String,
    orientation_csv: String,
    output_suffix: String,
    limit: i32,
    target_face_count: i32,
    texture_size: i32,
    force: bool,
    dry_run: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrellisBatchResult {
    command: String,
    status: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SourceImageTransform {
    #[serde(default)]
    rotate_degrees: f32,
    #[serde(default)]
    flip_horizontal: bool,
    #[serde(default)]
    flip_vertical: bool,
}

impl Default for SourceImageTransform {
    fn default() -> Self {
        Self {
            rotate_degrees: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SourceImageOrientationEntry {
    file_name: String,
    relative_path: String,
    #[serde(default)]
    absolute_path: String,
    transform: SourceImageTransform,
    #[serde(default)]
    asset_role: String,
    #[serde(default)]
    notes: String,
    updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct HandoffAuditEntry {
    timestamp: String,
    tool: String,
    action: String,
    target: String,
    summary: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceImageOrientationManifest {
    schema_version: String,
    updated_at: String,
    tool: String,
    source_folder: String,
    assets: BTreeMap<String, SourceImageOrientationEntry>,
    audit: Vec<HandoffAuditEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceImageOrientationCatalog {
    manifest_path: String,
    source_folder: String,
    assets: Vec<SourceImageOrientationEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceImageOrientationRequest {
    project_path: String,
    source_folder: String,
    manifest_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceImageOrientationUpdate {
    project_path: String,
    source_folder: String,
    manifest_path: String,
    relative_path: String,
    transform: SourceImageTransform,
    asset_role: String,
    notes: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceImagePreviewRequest {
    absolute_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelAbsolutePreviewRequest {
    project_path: String,
    absolute_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MachineToolConfig {
    id: String,
    label: String,
    kind: String,
    executable_path: String,
    url: String,
    working_directory: String,
    notes: String,
    enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MachineConfig {
    schema_version: String,
    computer_name: String,
    updated_at: String,
    tools: Vec<MachineToolConfig>,
    audit: Vec<HandoffAuditEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MachineConfigReadout {
    path: String,
    config: MachineConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathPickerRequest {
    kind: String,
    title: String,
    initial_path: String,
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

fn asset_forge_refs_dir(local_folder: &Path) -> PathBuf {
    local_folder.join("refs").join("assetForge")
}

fn asset_forge_logs_dir(local_folder: &Path) -> PathBuf {
    asset_forge_refs_dir(local_folder).join("logs")
}

fn project_audit_log_path(local_folder: &Path) -> PathBuf {
    asset_forge_logs_dir(local_folder).join("asset-forge-audit.jsonl")
}

fn ensure_asset_forge_refs(local_folder: &Path) -> Result<(), String> {
    fs::create_dir_all(asset_forge_logs_dir(local_folder)).map_err(|error| {
        format!(
            "Could not create Asset Forge refs folder {}: {error}",
            asset_forge_logs_dir(local_folder).display()
        )
    })
}

fn append_project_audit_log(local_folder: &Path, entry: &HandoffAuditEntry) -> Result<(), String> {
    ensure_asset_forge_refs(local_folder)?;
    let path = project_audit_log_path(local_folder);
    let serialized = serde_json::to_string(entry)
        .map_err(|error| format!("Could not serialize audit log entry: {error}"))?;
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("Could not open audit log {}: {error}", path.display()))?;
    writeln!(file, "{serialized}")
        .map_err(|error| format!("Could not write audit log {}: {error}", path.display()))
}

fn read_ship_model_manifest(path: &Path) -> Result<ShipModelOrientationManifest, String> {
    if !path.exists() {
        return Ok(ShipModelOrientationManifest {
            schema_version: SCHEMA_VERSION.to_string(),
            updated_at: now_isoish(),
            models: HashMap::new(),
            audit: Vec::new(),
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

fn resolve_project_file(project_path: &str, path: &str) -> Result<PathBuf, String> {
    let candidate = PathBuf::from(path.trim());
    if candidate.is_absolute() {
        return Ok(candidate);
    }

    Ok(local_folder_for_project(&PathBuf::from(project_path))?.join(candidate))
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

fn default_source_orientation_manifest(
    source_folder: &Path,
    manifest_path: &Path,
) -> SourceImageOrientationManifest {
    SourceImageOrientationManifest {
        schema_version: SCHEMA_VERSION.to_string(),
        updated_at: now_isoish(),
        tool: "source-image-orientation".to_string(),
        source_folder: source_folder.to_string_lossy().to_string(),
        assets: BTreeMap::new(),
        audit: vec![HandoffAuditEntry {
            timestamp: now_isoish(),
            tool: "source-image-orientation".to_string(),
            action: "created-manifest".to_string(),
            target: manifest_path.to_string_lossy().to_string(),
            summary: "Initialized source image orientation handoff manifest.".to_string(),
        }],
    }
}

fn read_source_orientation_manifest(
    source_folder: &Path,
    manifest_path: &Path,
) -> Result<SourceImageOrientationManifest, String> {
    if !manifest_path.exists() {
        return Ok(default_source_orientation_manifest(source_folder, manifest_path));
    }

    let contents = fs::read_to_string(manifest_path)
        .map_err(|error| format!("Could not read {}: {error}", manifest_path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("Could not parse {}: {error}", manifest_path.display()))
}

fn write_source_orientation_manifest(
    manifest_path: &Path,
    manifest: &SourceImageOrientationManifest,
) -> Result<(), String> {
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }
    let serialized = serde_json::to_string_pretty(manifest)
        .map_err(|error| format!("Could not serialize source image orientation manifest: {error}"))?;
    fs::write(manifest_path, format!("{serialized}\n"))
        .map_err(|error| format!("Could not write {}: {error}", manifest_path.display()))
}

fn asset_forge_root() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "Could not resolve Asset Forge repository root.".to_string())
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

fn model_opportunities(path: &Path, size: u64) -> Vec<String> {
    let mut opportunities = Vec::new();
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    if size > 10_000_000 {
        opportunities.push("large-glb-optimize-for-runtime".to_string());
    }
    if name.contains("-raw") {
        opportunities.push("raw-generated-model".to_string());
    }
    if !name.contains("optimized") {
        opportunities.push("missing-optimized-name".to_string());
    }

    opportunities
}

fn inspect_model_triangles(path: &Path) -> u64 {
    let output = Command::new(npx_executable())
        .args([
            "--yes",
            "@gltf-transform/cli",
            "inspect",
            &path.to_string_lossy(),
        ])
        .output();

    let Ok(output) = output else {
        return 0;
    };
    if !output.status.success() {
        return 0;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut in_meshes = false;
    let mut total = 0_u64;
    for line in text.lines() {
        if line.contains("MESHES") {
            in_meshes = true;
            continue;
        }
        if in_meshes && line.contains("MATERIALS") {
            break;
        }
        if !in_meshes || !line.contains("TRIANGLES") {
            continue;
        }

        let normalized = line
            .replace('\u{001b}', "")
            .replace("[90m", "")
            .replace("[39m", "")
            .replace("[31m", "");
        let parts: Vec<&str> = normalized.split('│').map(str::trim).collect();
        if let Some(value) = parts.get(5) {
            let digits = value.replace(',', "");
            if let Ok(count) = digits.parse::<u64>() {
                total = total.saturating_add(count);
            }
        }
    }

    total
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

fn model_path_from_res_path(
    local_folder: &Path,
    allowed_model_folder: &Path,
    model_path: &str,
) -> Result<PathBuf, String> {
    let relative = model_path
        .strip_prefix("res://")
        .ok_or_else(|| format!("Unsupported model path: {}", model_path))?;
    let candidate = local_folder.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    let canonical_model_root = allowed_model_folder.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve model folder {}: {}",
            allowed_model_folder.display(),
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
            "Model path is outside the selected model folder: {}",
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

fn machine_config_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("machine-config.json"))
}

fn current_computer_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-computer".to_string())
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

fn default_machine_config() -> MachineConfig {
    MachineConfig {
        schema_version: SCHEMA_VERSION.to_string(),
        computer_name: current_computer_name(),
        updated_at: now_isoish(),
        tools: vec![
            MachineToolConfig {
                id: "comfyui".to_string(),
                label: "ComfyUI".to_string(),
                kind: "http-service".to_string(),
                executable_path: String::new(),
                url: "http://127.0.0.1:8000".to_string(),
                working_directory: String::new(),
                notes: "Used by Trellis batch conversion.".to_string(),
                enabled: true,
            },
            MachineToolConfig {
                id: "blender".to_string(),
                label: "Blender".to_string(),
                kind: "desktop-app".to_string(),
                executable_path: String::new(),
                url: String::new(),
                working_directory: String::new(),
                notes: "Optional automation target for model transforms and optimization.".to_string(),
                enabled: false,
            },
            MachineToolConfig {
                id: "imagemagick".to_string(),
                label: "ImageMagick".to_string(),
                kind: "cli".to_string(),
                executable_path: String::new(),
                url: String::new(),
                working_directory: String::new(),
                notes: "Optional external image optimization backend.".to_string(),
                enabled: false,
            },
        ],
        audit: vec![HandoffAuditEntry {
            timestamp: now_isoish(),
            tool: "machine-config".to_string(),
            action: "created-default-config".to_string(),
            target: current_computer_name(),
            summary: "Initialized machine-local tool configuration.".to_string(),
        }],
    }
}

fn read_machine_config() -> Result<MachineConfig, String> {
    let path = machine_config_path()?;
    if !path.exists() {
        return Ok(default_machine_config());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let mut config: MachineConfig = serde_json::from_str(&contents)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))?;
    if config.computer_name.trim().is_empty() {
        config.computer_name = current_computer_name();
    }
    Ok(config)
}

fn write_machine_config(mut config: MachineConfig) -> Result<MachineConfigReadout, String> {
    let path = machine_config_path()?;
    config.computer_name = current_computer_name();
    config.updated_at = now_isoish();
    config.audit.push(HandoffAuditEntry {
        timestamp: config.updated_at.clone(),
        tool: "machine-config".to_string(),
        action: "saved-config".to_string(),
        target: config.computer_name.clone(),
        summary: format!("Saved {} configured tool entries.", config.tools.len()),
    });

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }
    let serialized = serde_json::to_string_pretty(&config)
        .map_err(|error| format!("Could not serialize machine config: {error}"))?;
    fs::write(&path, format!("{serialized}\n"))
        .map_err(|error| format!("Could not write {}: {error}", path.display()))?;

    Ok(MachineConfigReadout {
        path: path.to_string_lossy().to_string(),
        config,
    })
}

fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn powershell_executable() -> String {
    Command::new("pwsh")
        .args(["-NoProfile", "-Command", "$PSVersionTable.PSVersion.ToString()"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|_| "pwsh".to_string())
        .unwrap_or_else(|| "powershell".to_string())
}

fn npx_executable() -> String {
    if cfg!(windows) {
        "npx.cmd".to_string()
    } else {
        "npx".to_string()
    }
}

#[tauri::command]
fn list_ship_model_orientations(
    request: ShipModelOrientationRequest,
) -> Result<ShipModelOrientationCatalog, String> {
    let project_path = PathBuf::from(&request.project_path);
    let local_folder = local_folder_for_project(&project_path)?;
    ensure_asset_forge_refs(&local_folder)?;
    let model_folder = resolve_project_folder(&request.project_path, &request.model_folder)?;
    let manifest_path = resolve_project_file(&request.project_path, &request.manifest_path)?;
    let manifest = read_ship_model_manifest(&manifest_path)?;
    let mut files = Vec::new();
    collect_glb_files(&model_folder, &mut files)?;
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
        model_folder: model_folder.to_string_lossy().to_string(),
        models,
    })
}

#[tauri::command]
fn save_ship_model_orientation(
    update: ShipModelOrientationUpdate,
) -> Result<ShipModelOrientationCatalog, String> {
    let project_path = PathBuf::from(&update.project_path);
    let local_folder = local_folder_for_project(&project_path)?;
    ensure_asset_forge_refs(&local_folder)?;
    let manifest_path = resolve_project_file(&update.project_path, &update.manifest_path)?;
    let mut manifest = read_ship_model_manifest(&manifest_path)?;
    let now = now_isoish();
    let audit = HandoffAuditEntry {
        timestamp: now.clone(),
        tool: "model-orientation".to_string(),
        action: "saved-orientation".to_string(),
        target: update.model_path.clone(),
        summary: "Updated GLB model orientation metadata for downstream runtime/import tooling.".to_string(),
    };
    manifest.updated_at = now;
    manifest
        .models
        .insert(update.model_path, update.transform);
    manifest.audit.push(audit.clone());
    write_ship_model_manifest(&manifest_path, &manifest)?;
    append_project_audit_log(&local_folder, &audit)?;
    list_ship_model_orientations(ShipModelOrientationRequest {
        project_path: update.project_path,
        model_folder: update.model_folder,
        manifest_path: update.manifest_path,
    })
}

#[tauri::command]
fn load_ship_model_preview(request: ShipModelPreviewRequest) -> Result<Vec<u8>, String> {
    let project_path = PathBuf::from(&request.project_path);
    let local_folder = local_folder_for_project(&project_path)?;
    let model_folder = resolve_project_folder(&request.project_path, &request.model_folder)?;
    let model_path = model_path_from_res_path(&local_folder, &model_folder, &request.model_path)?;
    fs::read(&model_path).map_err(|error| {
        format!(
            "Unable to read model preview {}: {}",
            model_path.display(),
            error
        )
    })
}

#[tauri::command]
fn load_absolute_model_preview(request: ModelAbsolutePreviewRequest) -> Result<Vec<u8>, String> {
    let local_folder = local_folder_for_project(&PathBuf::from(&request.project_path))?;
    let candidate = PathBuf::from(&request.absolute_path);
    let canonical_local = local_folder.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve local project folder {}: {}",
            local_folder.display(),
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
    if !canonical_candidate.starts_with(&canonical_local) {
        return Err(format!(
            "Model path is outside the linked project folder: {}",
            candidate.display()
        ));
    }

    fs::read(&canonical_candidate).map_err(|error| {
        format!(
            "Unable to read model preview {}: {}",
            canonical_candidate.display(),
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
fn scan_model_assets(request: ModelScanRequest) -> Result<ModelScanResult, String> {
    let source_folder = resolve_project_folder(&request.project_path, &request.source_folder)?;
    if !source_folder.exists() {
        return Err(format!(
            "Model source folder does not exist: {}",
            source_folder.display()
        ));
    }

    let mut files = Vec::new();
    collect_glb_files(&source_folder, &mut files)?;
    files.sort();

    let mut assets = Vec::new();
    for path in files {
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("Could not read metadata for {}: {error}", path.display()))?;
        let triangle_count = inspect_model_triangles(&path);
        assets.push(ModelAssetEntry {
            absolute_path: path.to_string_lossy().to_string(),
            relative_path: relative_path_string(&source_folder, &path),
            file_name: path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default(),
            file_size_bytes: metadata.len(),
            triangle_count,
            opportunities: model_opportunities(&path, metadata.len()),
        });
    }

    Ok(ModelScanResult {
        source_folder: source_folder.to_string_lossy().to_string(),
        assets,
    })
}

#[tauri::command]
fn optimize_model_assets(request: ModelOptimizeRequest) -> Result<ModelOptimizeResult, String> {
    let local_folder = local_folder_for_project(&PathBuf::from(&request.project_path))?;
    ensure_asset_forge_refs(&local_folder)?;
    let source_folder = resolve_project_folder(&request.project_path, &request.source_folder)?;
    let staging_folder = resolve_project_folder(&request.project_path, &request.staging_folder)?;
    let manifest_path = resolve_project_file(&request.project_path, &request.manifest_path)?;
    if !source_folder.exists() {
        return Err(format!(
            "Model source folder does not exist: {}",
            source_folder.display()
        ));
    }

    fs::create_dir_all(&staging_folder).map_err(|error| {
        format!(
            "Could not create model staging folder {}: {error}",
            staging_folder.display()
        )
    })?;

    let mut files = Vec::new();
    collect_glb_files(&source_folder, &mut files)?;
    files.sort();

    let compression = request.compression.trim().to_ascii_lowercase();
    let texture_compress = request.texture_compress.trim().to_ascii_lowercase();
    let mut outputs = Vec::new();

    for path in files {
        let relative = path.strip_prefix(&source_folder).unwrap_or(&path);
        let output_path = staging_folder.join(relative);
        if output_path.exists() && !request.force {
            continue;
        }
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!("Could not create output folder {}: {error}", parent.display())
            })?;
        }

        let source_metadata = fs::metadata(&path)
            .map_err(|error| format!("Could not read metadata for {}: {error}", path.display()))?;
        let source_triangles = inspect_model_triangles(&path);
        let simplify_ratio = if request.simplify && request.target_triangles > 0 && source_triangles > 0 {
            ((request.target_triangles as f32) / (source_triangles as f32)).clamp(0.01, 1.0)
        } else {
            0.0
        };

        let mut args = vec![
            "--yes".to_string(),
            "@gltf-transform/cli".to_string(),
            "optimize".to_string(),
            path.to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string(),
        ];
        if !compression.is_empty() && compression != "none" && compression != "false" {
            args.push("--compress".to_string());
            args.push(compression.clone());
        }
        if !texture_compress.is_empty()
            && texture_compress != "none"
            && texture_compress != "false"
        {
            args.push("--texture-compress".to_string());
            args.push(texture_compress.clone());
        }
        args.push("--simplify".to_string());
        args.push(if request.simplify { "true" } else { "false" }.to_string());
        if request.simplify && simplify_ratio > 0.0 && simplify_ratio < 1.0 {
            args.push("--simplify-ratio".to_string());
            args.push(format!("{simplify_ratio:.3}"));
        }

        let output = Command::new(npx_executable())
            .args(&args)
            .output()
            .map_err(|error| format!("Could not launch glTF optimizer through npx: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "Model optimization failed for {}.\n{}\n{}",
                path.display(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output_metadata = fs::metadata(&output_path)
            .map_err(|error| format!("Could not read {}: {error}", output_path.display()))?;
        outputs.push(ModelOptimizeEntry {
            source_path: path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            source_size_bytes: source_metadata.len(),
            output_size_bytes: output_metadata.len(),
            source_triangles,
            target_triangles: request.target_triangles,
            simplify_ratio,
            action: "gltf-transform-optimize".to_string(),
            command: format!("npx {}", args.join(" ")),
        });
    }

    let now = now_isoish();
    let audit = HandoffAuditEntry {
        timestamp: now.clone(),
        tool: "model-optimization".to_string(),
        action: "optimized-models".to_string(),
        target: staging_folder.to_string_lossy().to_string(),
        summary: format!("Optimized {} GLB model assets to staging.", outputs.len()),
    };
    let manifest = ModelOptimizationManifest {
        schema_version: SCHEMA_VERSION.to_string(),
        updated_at: now,
        tool: "model-optimization".to_string(),
        source_folder: source_folder.to_string_lossy().to_string(),
        staging_folder: staging_folder.to_string_lossy().to_string(),
        outputs,
        audit: vec![audit.clone()],
    };
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }
    let serialized = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("Could not serialize model optimization manifest: {error}"))?;
    fs::write(&manifest_path, format!("{serialized}\n"))
        .map_err(|error| format!("Could not write {}: {error}", manifest_path.display()))?;
    append_project_audit_log(&local_folder, &audit)?;

    Ok(ModelOptimizeResult {
        staging_folder: manifest.staging_folder,
        manifest_path: manifest_path.to_string_lossy().to_string(),
        outputs: manifest.outputs,
    })
}

#[tauri::command]
fn list_source_image_orientations(
    request: SourceImageOrientationRequest,
) -> Result<SourceImageOrientationCatalog, String> {
    let local_folder = local_folder_for_project(&PathBuf::from(&request.project_path))?;
    ensure_asset_forge_refs(&local_folder)?;
    let source_folder = resolve_project_folder(&request.project_path, &request.source_folder)?;
    let manifest_path = resolve_project_folder(&request.project_path, &request.manifest_path)?;
    if !source_folder.exists() {
        return Err(format!(
            "Source image folder does not exist: {}",
            source_folder.display()
        ));
    }

    let manifest = read_source_orientation_manifest(&source_folder, &manifest_path)?;
    let mut files = Vec::new();
    collect_image_files(&source_folder, &mut files)?;
    files.sort();

    let assets = files
        .into_iter()
        .map(|path| {
            let relative_path = relative_path_string(&source_folder, &path);
            manifest
                .assets
                .get(&relative_path)
                .cloned()
                .unwrap_or_else(|| SourceImageOrientationEntry {
                    file_name: path
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| relative_path.clone()),
                    relative_path: relative_path.clone(),
                    absolute_path: path.to_string_lossy().to_string(),
                    transform: SourceImageTransform::default(),
                    asset_role: String::new(),
                    notes: String::new(),
                    updated_at: String::new(),
                })
        })
        .map(|mut entry| {
            if entry.absolute_path.trim().is_empty() {
                entry.absolute_path = source_folder.join(&entry.relative_path).to_string_lossy().to_string();
            }
            entry
        })
        .collect();

    Ok(SourceImageOrientationCatalog {
        manifest_path: manifest_path.to_string_lossy().to_string(),
        source_folder: source_folder.to_string_lossy().to_string(),
        assets,
    })
}

#[tauri::command]
fn save_source_image_orientation(
    update: SourceImageOrientationUpdate,
) -> Result<SourceImageOrientationCatalog, String> {
    let local_folder = local_folder_for_project(&PathBuf::from(&update.project_path))?;
    ensure_asset_forge_refs(&local_folder)?;
    let source_folder = resolve_project_folder(&update.project_path, &update.source_folder)?;
    let manifest_path = resolve_project_folder(&update.project_path, &update.manifest_path)?;
    let mut manifest = read_source_orientation_manifest(&source_folder, &manifest_path)?;
    let now = now_isoish();
    let absolute_path = source_folder.join(&update.relative_path);
    let file_name = absolute_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| update.relative_path.clone());

    manifest.updated_at = now.clone();
    manifest.source_folder = source_folder.to_string_lossy().to_string();
    manifest.assets.insert(
        update.relative_path.clone(),
        SourceImageOrientationEntry {
            file_name,
            relative_path: update.relative_path.clone(),
            absolute_path: absolute_path.to_string_lossy().to_string(),
            transform: update.transform,
            asset_role: update.asset_role,
            notes: update.notes,
            updated_at: now.clone(),
        },
    );
    let audit = HandoffAuditEntry {
        timestamp: now,
        tool: "source-image-orientation".to_string(),
        action: "saved-orientation".to_string(),
        target: update.relative_path,
        summary: "Updated source image orientation metadata for downstream tools and game integration.".to_string(),
    };
    manifest.audit.push(audit.clone());
    write_source_orientation_manifest(&manifest_path, &manifest)?;
    append_project_audit_log(&local_folder, &audit)?;

    list_source_image_orientations(SourceImageOrientationRequest {
        project_path: update.project_path,
        source_folder: update.source_folder,
        manifest_path: update.manifest_path,
    })
}

#[tauri::command]
fn load_source_image_preview(request: SourceImagePreviewRequest) -> Result<Vec<u8>, String> {
    let path = PathBuf::from(request.absolute_path);
    fs::read(&path).map_err(|error| format!("Could not read image {}: {error}", path.display()))
}

#[tauri::command]
fn get_machine_config() -> Result<MachineConfigReadout, String> {
    let path = machine_config_path()?;
    Ok(MachineConfigReadout {
        path: path.to_string_lossy().to_string(),
        config: read_machine_config()?,
    })
}

#[tauri::command]
fn save_machine_config(config: MachineConfig) -> Result<MachineConfigReadout, String> {
    write_machine_config(config)
}

#[tauri::command]
fn pick_path(request: PathPickerRequest) -> Result<Option<String>, String> {
    let title = powershell_single_quoted(if request.title.trim().is_empty() {
        "Select path"
    } else {
        request.title.trim()
    });
    let initial_path = request.initial_path.trim();
    let initial = if initial_path.is_empty() {
        String::new()
    } else {
        let path = PathBuf::from(initial_path);
        let initial_dir = if path.is_dir() {
            path
        } else {
            path.parent().map(Path::to_path_buf).unwrap_or(path)
        };
        initial_dir.to_string_lossy().to_string()
    };
    let initial = powershell_single_quoted(&initial);
    let script = if request.kind.eq_ignore_ascii_case("folder") {
        format!(
            "Add-Type -AssemblyName System.Windows.Forms; $dialog = New-Object System.Windows.Forms.FolderBrowserDialog; $dialog.Description = {title}; if ({initial}.Length -gt 0) {{ $dialog.SelectedPath = {initial} }}; if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ $dialog.SelectedPath }}"
        )
    } else {
        format!(
            "Add-Type -AssemblyName System.Windows.Forms; $dialog = New-Object System.Windows.Forms.OpenFileDialog; $dialog.Title = {title}; $dialog.CheckFileExists = $false; if ({initial}.Length -gt 0) {{ $dialog.InitialDirectory = {initial} }}; if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ $dialog.FileName }}"
        )
    };

    let shell = powershell_executable();
    let output = Command::new(&shell)
        .args(["-NoProfile", "-STA", "-Command", &script])
        .output()
        .map_err(|error| format!("Could not open path picker: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

#[tauri::command]
fn run_trellis_batch_conversion(request: TrellisBatchRequest) -> Result<TrellisBatchResult, String> {
    let script_path = asset_forge_root()?
        .join("tools")
        .join("Convert-Trellis2Batch.ps1");
    if !script_path.exists() {
        return Err(format!(
            "Trellis batch script not found: {}",
            script_path.display()
        ));
    }

    let source_folder = resolve_project_folder(&request.project_path, &request.source_folder)?;
    let final_output_folder =
        resolve_project_folder(&request.project_path, &request.final_output_folder)?;
    let workflow_path = resolve_project_folder(&request.project_path, &request.workflow_path)?;
    let comfy_output_folder =
        resolve_project_folder(&request.project_path, &request.comfy_output_folder)?;
    let staging_folder = if request.staging_folder.trim().is_empty() {
        None
    } else {
        Some(resolve_project_folder(
            &request.project_path,
            &request.staging_folder,
        )?)
    };
    let orientation_csv = if request.orientation_csv.trim().is_empty() {
        None
    } else {
        Some(resolve_project_folder(
            &request.project_path,
            &request.orientation_csv,
        )?)
    };

    let mut args = vec![
        "-NoProfile".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-File".to_string(),
        script_path.to_string_lossy().to_string(),
        "-SourceDir".to_string(),
        source_folder.to_string_lossy().to_string(),
        "-FinalOutputDir".to_string(),
        final_output_folder.to_string_lossy().to_string(),
        "-WorkflowPath".to_string(),
        workflow_path.to_string_lossy().to_string(),
        "-ComfyUrl".to_string(),
        request.comfy_url,
        "-ComfyOutputDir".to_string(),
        comfy_output_folder.to_string_lossy().to_string(),
        "-TargetFaceCount".to_string(),
        request.target_face_count.clamp(1_000, 1_000_000).to_string(),
        "-TextureSize".to_string(),
        request.texture_size.clamp(256, 4096).to_string(),
    ];

    if let Some(staging_folder) = staging_folder {
        args.push("-StagingDir".to_string());
        args.push(staging_folder.to_string_lossy().to_string());
    }
    if let Some(orientation_csv) = orientation_csv {
        args.push("-OrientationCsv".to_string());
        args.push(orientation_csv.to_string_lossy().to_string());
    }
    if !request.output_suffix.trim().is_empty() {
        args.push("-OutputSuffix".to_string());
        args.push(request.output_suffix);
    }
    if request.limit > 0 {
        args.push("-Limit".to_string());
        args.push(request.limit.to_string());
    }
    if request.force {
        args.push("-Force".to_string());
    }
    if request.dry_run {
        args.push("-WhatIfOnly".to_string());
    }

    let shell = powershell_executable();
    let output = Command::new(&shell)
        .args(&args)
        .output()
        .map_err(|error| format!("Could not launch PowerShell Trellis batch: {error}"))?;

    let status = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!(
            "Trellis batch failed with status {status}.\n{stdout}\n{stderr}"
        ));
    }

    Ok(TrellisBatchResult {
        command: format!("{shell} {}", args.join(" ")),
        status,
        stdout,
        stderr,
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
            load_absolute_model_preview,
            scan_image_assets,
            optimize_image_assets,
            scan_model_assets,
            optimize_model_assets,
            list_source_image_orientations,
            save_source_image_orientation,
            load_source_image_preview,
            get_machine_config,
            save_machine_config,
            pick_path,
            run_trellis_batch_conversion,
            close_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running Asset Forge");
}
