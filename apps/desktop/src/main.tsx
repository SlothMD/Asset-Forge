import { invoke } from "@tauri-apps/api/core";
import { FormEvent, StrictMode, useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import "./styles.css";

type ProjectSummary = {
  projectId: string;
  name: string;
  path: string;
  githubRepositoryUrl: string;
  localFolder: string;
  createdAt: string;
  updatedAt: string;
};

type ProjectFile = {
  schemaVersion: string;
  projectId: string;
  name: string;
  githubRepositoryUrl: string;
  description: string;
  notes: string;
  createdAt: string;
  updatedAt: string;
};

type OpenProject = {
  project: ProjectSummary;
  projectJson: ProjectFile;
};

type ProjectDetailsUpdate = {
  path: string;
  description: string;
  notes: string;
};

type ProjectLinksUpdate = {
  path: string;
  githubRepositoryUrl: string;
  localFolder: string;
};

type SyncAction = "status" | "pull" | "push";

type ProjectSyncResult = {
  command: string;
  output: string;
};

type ActiveTool = "steam-launch-prep" | "model-orientation" | "image-optimization" | null;

type ShipModelTransform = {
  yawDegrees: number;
  pitchDegrees: number;
  rollDegrees: number;
  scale: number;
};

type ShipModelOrientationEntry = {
  modelPath: string;
  fileName: string;
  absolutePath: string;
  transform: ShipModelTransform;
};

type ShipModelOrientationCatalog = {
  manifestPath: string;
  models: ShipModelOrientationEntry[];
};

type ImageAssetEntry = {
  absolutePath: string;
  relativePath: string;
  fileName: string;
  extension: string;
  inferredType: string;
  width: number;
  height: number;
  fileSizeBytes: number;
  opportunities: string[];
};

type ImageScanResult = {
  sourceFolder: string;
  assets: ImageAssetEntry[];
};

type ImageOptimizeEntry = {
  sourcePath: string;
  outputPath: string;
  sourceSizeBytes: number;
  outputSizeBytes: number;
  sourceWidth: number;
  sourceHeight: number;
  outputWidth: number;
  outputHeight: number;
  action: string;
};

type ImageOptimizeResult = {
  stagingFolder: string;
  outputs: ImageOptimizeEntry[];
};

const fallbackProjectKey = "asset-forge-projects";
const fallbackProjectFilePrefix = "asset-forge-project-file:";

function isTauriRuntime() {
  return "__TAURI_INTERNALS__" in window;
}

function createFallbackProject(name: string): OpenProject {
  const timestamp = Date.now();
  const cleanName = name.trim();
  const projectId = `proj_${cleanName
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")}_${timestamp}`;
  const now = new Date(timestamp).toISOString();
  const project: ProjectSummary = {
    projectId,
    name: cleanName,
    path: `browser-local://${projectId}`,
    githubRepositoryUrl: "",
    localFolder: "",
    createdAt: now,
    updatedAt: now
  };
  const projectJson: ProjectFile = {
    schemaVersion: "0.1.0",
    projectId,
    name: cleanName,
    githubRepositoryUrl: "",
    description: "",
    notes: "",
    createdAt: now,
    updatedAt: now
  };

  return { project, projectJson };
}

function readFallbackProjects(): ProjectSummary[] {
  const stored = window.localStorage.getItem(fallbackProjectKey);
  if (!stored) return [];

  try {
    return JSON.parse(stored) as ProjectSummary[];
  } catch {
    return [];
  }
}

function writeFallbackProjects(projects: ProjectSummary[]) {
  window.localStorage.setItem(fallbackProjectKey, JSON.stringify(projects));
}

function readFallbackProjectFile(project: ProjectSummary): ProjectFile {
  const stored = window.localStorage.getItem(`${fallbackProjectFilePrefix}${project.projectId}`);
  if (stored) {
    try {
      return JSON.parse(stored) as ProjectFile;
    } catch {
      // Fall through to a compatible shell.
    }
  }

  return {
    schemaVersion: "0.1.0",
    projectId: project.projectId,
    name: project.name,
    githubRepositoryUrl: project.githubRepositoryUrl ?? "",
    description: "",
    notes: "",
    createdAt: project.createdAt,
    updatedAt: project.updatedAt
  };
}

function writeFallbackProjectFile(project: ProjectFile) {
  window.localStorage.setItem(
    `${fallbackProjectFilePrefix}${project.projectId}`,
    JSON.stringify(project)
  );
}

const projectApi = {
  async listProjects(): Promise<ProjectSummary[]> {
    if (isTauriRuntime()) {
      return invoke<ProjectSummary[]>("list_projects");
    }

    return readFallbackProjects();
  },

  async createProject(name: string): Promise<OpenProject> {
    if (isTauriRuntime()) {
      return invoke<OpenProject>("create_project", { name });
    }

    const opened = createFallbackProject(name);
    writeFallbackProjectFile(opened.projectJson);
    writeFallbackProjects([opened.project, ...readFallbackProjects()]);
    return opened;
  },

  async openProject(project: ProjectSummary): Promise<OpenProject> {
    if (isTauriRuntime()) {
      return invoke<OpenProject>("open_project", { path: project.path });
    }

    return {
      project,
      projectJson: readFallbackProjectFile(project)
    };
  },

  async updateProjectDetails(update: ProjectDetailsUpdate): Promise<OpenProject> {
    if (isTauriRuntime()) {
      return invoke<OpenProject>("update_project_details", { update });
    }

    const projects = readFallbackProjects();
    const project = projects.find((candidate) => candidate.path === update.path);
    if (!project) {
      throw new Error("Project not found.");
    }

    const updatedAt = new Date().toISOString();
    const projectJson = readFallbackProjectFile(project);
    const updatedProject: ProjectSummary = { ...project, updatedAt };
    const updatedJson: ProjectFile = {
      ...projectJson,
      githubRepositoryUrl: project.githubRepositoryUrl ?? projectJson.githubRepositoryUrl ?? "",
      description: update.description,
      notes: update.notes,
      updatedAt
    };

    writeFallbackProjectFile(updatedJson);
    writeFallbackProjects(
      projects
        .map((candidate) =>
          candidate.projectId === updatedProject.projectId ? updatedProject : candidate
        )
        .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
    );

    return { project: updatedProject, projectJson: updatedJson };
  },

  async updateProjectLinks(update: ProjectLinksUpdate): Promise<OpenProject> {
    if (isTauriRuntime()) {
      return invoke<OpenProject>("update_project_links", { update });
    }

    const projects = readFallbackProjects();
    const project = projects.find((candidate) => candidate.path === update.path);
    if (!project) {
      throw new Error("Project not found.");
    }

    const updatedAt = new Date().toISOString();
    const projectJson = readFallbackProjectFile(project);
    const updatedProject: ProjectSummary = {
      ...project,
      githubRepositoryUrl: update.githubRepositoryUrl,
      localFolder: update.localFolder,
      updatedAt
    };
    const updatedJson: ProjectFile = {
      ...projectJson,
      githubRepositoryUrl: update.githubRepositoryUrl,
      updatedAt
    };

    writeFallbackProjectFile(updatedJson);
    writeFallbackProjects(
      projects
        .map((candidate) =>
          candidate.projectId === updatedProject.projectId ? updatedProject : candidate
        )
        .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
    );

    return { project: updatedProject, projectJson: updatedJson };
  },

  async syncExternalProject(path: string, action: SyncAction): Promise<ProjectSyncResult> {
    if (isTauriRuntime()) {
      return invoke<ProjectSyncResult>("sync_external_project", {
        request: { path, action }
      });
    }

    return {
      command: `git ${action}`,
      output: "Git sync is available in the desktop runtime."
    };
  },

  async listShipModelOrientations(projectPath: string): Promise<ShipModelOrientationCatalog> {
    if (isTauriRuntime()) {
      return invoke<ShipModelOrientationCatalog>("list_ship_model_orientations", {
        projectPath
      });
    }

    return {
      manifestPath: "Desktop runtime required",
      models: []
    };
  },

  async saveShipModelOrientation(
    projectPath: string,
    modelPath: string,
    transform: ShipModelTransform
  ): Promise<ShipModelOrientationCatalog> {
    if (isTauriRuntime()) {
      return invoke<ShipModelOrientationCatalog>("save_ship_model_orientation", {
        update: { projectPath, modelPath, transform }
      });
    }

    return {
      manifestPath: "Desktop runtime required",
      models: []
    };
  },

  async loadShipModelPreview(projectPath: string, modelPath: string): Promise<ArrayBuffer> {
    if (isTauriRuntime()) {
      const bytes = await invoke<number[]>("load_ship_model_preview", {
        request: { projectPath, modelPath }
      });
      return new Uint8Array(bytes).buffer;
    }

    throw new Error("3D preview is available in the desktop runtime.");
  },

  async scanImageAssets(projectPath: string, sourceFolder: string): Promise<ImageScanResult> {
    if (isTauriRuntime()) {
      return invoke<ImageScanResult>("scan_image_assets", {
        request: { projectPath, sourceFolder }
      });
    }

    return { sourceFolder, assets: [] };
  },

  async optimizeImageAssets(
    projectPath: string,
    sourceFolder: string,
    stagingFolder: string,
    targetMaxDimension: number,
    outputFormat: string,
    jpegQuality: number
  ): Promise<ImageOptimizeResult> {
    if (isTauriRuntime()) {
      return invoke<ImageOptimizeResult>("optimize_image_assets", {
        request: {
          projectPath,
          sourceFolder,
          stagingFolder,
          targetMaxDimension,
          outputFormat,
          jpegQuality
        }
      });
    }

    return { stagingFolder, outputs: [] };
  }
};

function formatDate(value: string) {
  const numericTimestamp = Number(value.replace(/Z$/, ""));
  const date = Number.isFinite(numericTimestamp) ? new Date(numericTimestamp) : new Date(value);

  if (Number.isNaN(date.getTime())) return value;

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short"
  }).format(date);
}

function formatBytes(value: number) {
  if (!Number.isFinite(value) || value <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let amount = value;
  let unitIndex = 0;
  while (amount >= 1024 && unitIndex < units.length - 1) {
    amount /= 1024;
    unitIndex += 1;
  }

  return `${amount.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}

async function closeApp() {
  if (isTauriRuntime()) {
    await invoke("close_app");
    return;
  }

  window.close();
}

function App() {
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [activeProject, setActiveProject] = useState<OpenProject | null>(null);
  const [activeTool, setActiveTool] = useState<ActiveTool>(null);
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [isProjectEditorOpen, setIsProjectEditorOpen] = useState(false);
  const [isBusy, setIsBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [projectName, setProjectName] = useState("");
  const [description, setDescription] = useState("");
  const [notes, setNotes] = useState("");
  const [githubRepositoryUrl, setGithubRepositoryUrl] = useState("");
  const [localFolder, setLocalFolder] = useState("");
  const [syncOutput, setSyncOutput] = useState<ProjectSyncResult | null>(null);

  const storageMode = useMemo(
    () => (isTauriRuntime() ? "Local project folders" : "Browser preview storage"),
    []
  );

  async function refreshProjects() {
    setError(null);
    const nextProjects = await projectApi.listProjects();
    setProjects(nextProjects);
    return nextProjects;
  }

  async function openProject(project: ProjectSummary) {
    setIsBusy(true);
    setError(null);

    try {
      const opened = await projectApi.openProject(project);
      setActiveProject(opened);
      setActiveTool(null);
      setIsProjectEditorOpen(false);
      setDescription(opened.projectJson.description ?? "");
      setNotes(opened.projectJson.notes ?? "");
      setGithubRepositoryUrl(opened.projectJson.githubRepositoryUrl ?? "");
      setLocalFolder(opened.project.localFolder ?? "");
      setSyncOutput(null);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setIsBusy(false);
    }
  }

  useEffect(() => {
    refreshProjects()
      .then((nextProjects) => {
        if (nextProjects[0]) {
          return openProject(nextProjects[0]);
        }
      })
      .catch((caught: unknown) =>
        setError(caught instanceof Error ? caught.message : String(caught))
      )
      .finally(() => setIsBusy(false));
  }, []);

  async function handleCreateProject(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const name = projectName.trim();
    if (!name) {
      setError("Project name is required.");
      return;
    }

    setIsBusy(true);
    setError(null);

    try {
      const opened = await projectApi.createProject(name);
      setProjectName("");
      setIsCreateOpen(false);
      setActiveProject(opened);
      setActiveTool(null);
      setDescription("");
      setNotes("");
      setGithubRepositoryUrl("");
      setLocalFolder("");
      setSyncOutput(null);
      await refreshProjects();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setIsBusy(false);
    }
  }

  async function handleSaveDetails(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!activeProject) return;

    setIsBusy(true);
    setError(null);

    try {
      const opened = await projectApi.updateProjectDetails({
        path: activeProject.project.path,
        description,
        notes
      });
      setActiveProject(opened);
      await refreshProjects();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setIsBusy(false);
    }
  }

  async function handleSaveLinks(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!activeProject) return;

    setIsBusy(true);
    setError(null);

    try {
      const opened = await projectApi.updateProjectLinks({
        path: activeProject.project.path,
        githubRepositoryUrl,
        localFolder
      });
      setActiveProject(opened);
      setGithubRepositoryUrl(opened.projectJson.githubRepositoryUrl ?? "");
      setLocalFolder(opened.project.localFolder ?? "");
      await refreshProjects();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setIsBusy(false);
    }
  }

  async function handleSyncExternalProject(action: SyncAction) {
    if (!activeProject) return;

    setIsBusy(true);
    setError(null);
    setSyncOutput(null);

    try {
      const result = await projectApi.syncExternalProject(activeProject.project.path, action);
      setSyncOutput(result);
      if (action === "pull") {
        const opened = await projectApi.openProject(activeProject.project);
        setActiveProject(opened);
      }
      await refreshProjects();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <main className="app-shell">
      <aside className="project-rail">
        <header className="rail-header">
          <div>
            <p className="eyebrow">Asset Forge</p>
            <h1>Projects</h1>
          </div>
          <div className="rail-actions" aria-label="Project actions">
            <button
              type="button"
              className="rail-action-button"
              aria-label="Add new project"
              title="Add new project"
              onClick={() => setIsCreateOpen(true)}
            >
              +
            </button>
            <button
              type="button"
              className="rail-action-button"
              aria-label="Settings"
              title="Settings"
              onClick={() => setError("Settings are not implemented yet.")}
            >
              ⚙
            </button>
            <button
              type="button"
              className="rail-action-button"
              aria-label="Close Asset Forge"
              title="Close Asset Forge"
              onClick={() => void closeApp()}
            >
              x
            </button>
          </div>
        </header>

        <div className="rail-meta">
          <span>{projects.length} projects</span>
          <span>{storageMode}</span>
        </div>

        <div className="project-list">
          {projects.map((project) => (
            <button
              key={project.projectId}
              type="button"
              className={
                activeProject?.project.projectId === project.projectId
                  ? "project-row active"
                  : "project-row"
              }
              onClick={() => void openProject(project)}
            >
              <strong>{project.name}</strong>
              <span>{formatDate(project.updatedAt)}</span>
            </button>
          ))}
          {!isBusy && projects.length === 0 ? (
            <p className="empty-state">No projects yet. Use + to create one.</p>
          ) : null}
        </div>
      </aside>

      <section className="detail-shell">
        {error ? <div className="error-banner">{error}</div> : null}

        {activeProject ? (
          <>
            <header className="project-header">
              <div>
                <p className="eyebrow">Launcher</p>
                <h2>{activeProject.project.name}</h2>
                <p className="path-line">{activeProject.project.path}</p>
              </div>
              <div className="project-actions">
                <button type="button" onClick={() => setIsProjectEditorOpen(true)}>
                  Edit
                </button>
                <select
                  aria-label="Project tools"
                  value={activeTool ?? ""}
                  onChange={(event) =>
                    setActiveTool(
                      event.target.value === "steam-launch-prep"
                        ? "steam-launch-prep"
                        : event.target.value === "model-orientation"
                          ? "model-orientation"
                          : event.target.value === "image-optimization"
                            ? "image-optimization"
                          : null
                    )
                  }
                >
                  <option value="">Tools</option>
                  <option value="model-orientation">Model Orientation</option>
                  <option value="image-optimization">Image Optimization</option>
                  <option value="steam-launch-prep">Steam Launch Prep</option>
                </select>
              </div>
            </header>

            <div className="project-body tool-only">
              <section className="detail-card tool-card">
                {activeTool === "steam-launch-prep" ? (
                  <SteamLaunchPrep project={activeProject.project} />
                ) : activeTool === "model-orientation" ? (
                  <ModelOrientationTool project={activeProject.project} />
                ) : activeTool === "image-optimization" ? (
                  <ImageOptimizationTool project={activeProject.project} />
                ) : (
                  <div className="tool-empty">
                    <h3>Tools</h3>
                    <p>Select a project tool from the dropdown.</p>
                  </div>
                )}
              </section>
            </div>
          </>
        ) : (
          <div className="no-project">
            <p className="eyebrow">Launcher</p>
            <h2>No Project Open</h2>
            <p>Create a project with + or select an existing project from the left rail.</p>
          </div>
        )}
      </section>

      {isCreateOpen ? (
        <div className="modal-backdrop" role="presentation" onMouseDown={() => setIsCreateOpen(false)}>
          <section
            className="modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="create-project-title"
            onMouseDown={(event) => event.stopPropagation()}
          >
            <header className="modal-header">
              <h2 id="create-project-title">Add Project</h2>
              <button
                type="button"
                className="icon-button subtle"
                aria-label="Close"
                onClick={() => setIsCreateOpen(false)}
              >
                x
              </button>
            </header>
            <form className="create-form" onSubmit={(event) => void handleCreateProject(event)}>
              <label htmlFor="project-name">Project name</label>
              <input
                id="project-name"
                autoFocus
                value={projectName}
                placeholder="Sector 404"
                onChange={(event) => setProjectName(event.target.value)}
              />
              <button type="submit" className="primary" disabled={isBusy}>
                Create
              </button>
            </form>
          </section>
        </div>
      ) : null}

      {activeProject && isProjectEditorOpen ? (
        <div
          className="modal-backdrop"
          role="presentation"
          onMouseDown={() => setIsProjectEditorOpen(false)}
        >
          <section
            className="modal project-editor-modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="project-editor-title"
            onMouseDown={(event) => event.stopPropagation()}
          >
            <header className="modal-header">
              <div>
                <p className="eyebrow">Project</p>
                <h2 id="project-editor-title">{activeProject.project.name}</h2>
              </div>
              <button
                type="button"
                className="icon-button subtle"
                aria-label="Close"
                onClick={() => setIsProjectEditorOpen(false)}
              >
                x
              </button>
            </header>

            <div className="project-editor-grid">
              <section className="editor-panel">
                <div className="section-heading">
                  <h3>Project Details</h3>
                  <span>{activeProject.projectJson.schemaVersion}</span>
                </div>
                <form className="edit-form" onSubmit={(event) => void handleSaveDetails(event)}>
                  <label>
                    Description
                    <textarea
                      value={description}
                      rows={4}
                      onChange={(event) => setDescription(event.target.value)}
                    />
                  </label>
                  <label>
                    Notes
                    <textarea
                      value={notes}
                      rows={7}
                      onChange={(event) => setNotes(event.target.value)}
                    />
                  </label>
                  <div className="readout-grid compact">
                    <div>
                      <dt>Created</dt>
                      <dd>{formatDate(activeProject.project.createdAt)}</dd>
                    </div>
                    <div>
                      <dt>Updated</dt>
                      <dd>{formatDate(activeProject.project.updatedAt)}</dd>
                    </div>
                  </div>
                  <button type="submit" className="primary" disabled={isBusy}>
                    Save Details
                  </button>
                </form>
              </section>

              <section className="editor-panel">
                <div className="section-heading">
                  <h3>External Project</h3>
                  <span>machine local</span>
                </div>
                <form className="link-form" onSubmit={(event) => void handleSaveLinks(event)}>
                  <label>
                    GitHub repository
                    <input
                      value={githubRepositoryUrl}
                      placeholder="https://github.com/owner/repository"
                      onChange={(event) => setGithubRepositoryUrl(event.target.value)}
                    />
                  </label>
                  <label>
                    Local folder on this machine
                    <input
                      value={localFolder}
                      placeholder="C:\Projects\ExternalGame"
                      onChange={(event) => setLocalFolder(event.target.value)}
                    />
                  </label>
                  <div className="button-row">
                    <button type="submit" className="primary" disabled={isBusy}>
                      Save Links
                    </button>
                    <button
                      type="button"
                      disabled={isBusy || !localFolder}
                      onClick={() => void handleSyncExternalProject("status")}
                    >
                      Git Status
                    </button>
                    <button
                      type="button"
                      disabled={isBusy || !localFolder}
                      onClick={() => void handleSyncExternalProject("pull")}
                    >
                      Pull
                    </button>
                    <button
                      type="button"
                      disabled={isBusy || !localFolder}
                      onClick={() => void handleSyncExternalProject("push")}
                    >
                      Push
                    </button>
                  </div>
                </form>

                <div className="readout-grid compact">
                  <div>
                    <dt>Portable hook</dt>
                    <dd>{activeProject.projectJson.githubRepositoryUrl || "No GitHub repository set."}</dd>
                  </div>
                  <div>
                    <dt>This machine</dt>
                    <dd>{activeProject.project.localFolder || "No local folder bound on this machine."}</dd>
                  </div>
                </div>

                {syncOutput ? (
                  <pre className="sync-output">
                    <strong>{syncOutput.command}</strong>
                    {"\n"}
                    {syncOutput.output}
                  </pre>
                ) : null}
              </section>
            </div>
          </section>
        </div>
      ) : null}
    </main>
  );
}

function SteamLaunchPrep({ project }: { project: ProjectSummary }) {
  return (
    <div className="steam-tool">
      <div className="section-heading">
        <h3>Steam Launch Prep</h3>
        <span>draft</span>
      </div>
      <div className="prep-grid">
        <label>
          Steam app name
          <input defaultValue={project.name} />
        </label>
        <label>
          Short description
          <textarea rows={3} placeholder="One-sentence store hook." />
        </label>
        <label>
          Capsule asset notes
          <textarea rows={4} placeholder="Key art, logo, color, and readability requirements." />
        </label>
        <label>
          Launch checklist
          <textarea
            rows={7}
            defaultValue={[
              "Store page copy",
              "Capsule art",
              "Trailer or gameplay clip",
              "Screenshots",
              "System requirements",
              "Tags and categories",
              "Launch build candidate"
            ].join("\n")}
          />
        </label>
      </div>
    </div>
  );
}

function ImageOptimizationTool({ project }: { project: ProjectSummary }) {
  const [sourceFolder, setSourceFolder] = useState("assets/textures");
  const [stagingFolder, setStagingFolder] = useState("tmp/asset-forge-optimized-textures");
  const [targetMaxDimension, setTargetMaxDimension] = useState(2048);
  const [outputFormat, setOutputFormat] = useState("jpg");
  const [jpegQuality, setJpegQuality] = useState(82);
  const [scan, setScan] = useState<ImageScanResult | null>(null);
  const [result, setResult] = useState<ImageOptimizeResult | null>(null);
  const [status, setStatus] = useState("");
  const [isBusy, setIsBusy] = useState(false);

  const totalBytes = scan?.assets.reduce((sum, asset) => sum + asset.fileSizeBytes, 0) ?? 0;
  const opportunityCount =
    scan?.assets.reduce((sum, asset) => sum + asset.opportunities.length, 0) ?? 0;
  const outputBytes = result?.outputs.reduce((sum, output) => sum + output.outputSizeBytes, 0) ?? 0;
  const sourceBytes =
    result?.outputs.reduce((sum, output) => sum + output.sourceSizeBytes, 0) ?? 0;

  async function runScan() {
    setIsBusy(true);
    setStatus("");
    setResult(null);
    try {
      const next = await projectApi.scanImageAssets(project.path, sourceFolder);
      setScan(next);
      setStatus(`Scanned ${next.assets.length} image assets.`);
    } catch (caught) {
      setStatus(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setIsBusy(false);
    }
  }

  async function runOptimize() {
    setIsBusy(true);
    setStatus("");
    try {
      const next = await projectApi.optimizeImageAssets(
        project.path,
        sourceFolder,
        stagingFolder,
        targetMaxDimension,
        outputFormat,
        jpegQuality
      );
      setResult(next);
      setStatus(`Wrote ${next.outputs.length} staged image assets.`);
    } catch (caught) {
      setStatus(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <div className="image-tool">
      <div className="section-heading">
        <h3>Image Optimization</h3>
        <span>{scan ? `${scan.assets.length} assets` : "scanner"}</span>
      </div>

      <p className="tool-note">
        Scan a directory recursively, classify image files, and write optimized copies to a
        staging folder. Source files are not modified.
      </p>

      <div className="image-config-grid">
        <label>
          Source folder
          <input
            value={sourceFolder}
            placeholder="assets/textures or D:\Projects\Game\assets\textures"
            onChange={(event) => setSourceFolder(event.target.value)}
          />
        </label>
        <label>
          Staging folder
          <input
            value={stagingFolder}
            placeholder="tmp/asset-forge-optimized-textures"
            onChange={(event) => setStagingFolder(event.target.value)}
          />
        </label>
        <label>
          Type preset
          <select defaultValue="runtime-pbr">
            <option value="runtime-pbr">Runtime PBR</option>
            <option value="ui">UI/Icon</option>
            <option value="marketing">Marketing</option>
            <option value="mixed">Mixed/Review</option>
          </select>
        </label>
        <label>
          Max dimension
          <input
            type="number"
            min="256"
            max="8192"
            step="256"
            value={targetMaxDimension}
            onChange={(event) => setTargetMaxDimension(Number(event.target.value) || 2048)}
          />
        </label>
        <label>
          Output format
          <select value={outputFormat} onChange={(event) => setOutputFormat(event.target.value)}>
            <option value="jpg">JPG</option>
            <option value="png">PNG</option>
          </select>
        </label>
        <label>
          JPG quality
          <input
            type="number"
            min="35"
            max="100"
            value={jpegQuality}
            onChange={(event) => setJpegQuality(Number(event.target.value) || 82)}
          />
        </label>
      </div>

      <div className="button-row">
        <button type="button" className="primary" disabled={isBusy} onClick={() => void runScan()}>
          Scan Images
        </button>
        <button
          type="button"
          disabled={isBusy || !scan || scan.assets.length === 0}
          onClick={() => void runOptimize()}
        >
          Optimize To Staging
        </button>
      </div>

      {scan ? (
        <div className="scan-summary">
          <div>
            <dt>Total source size</dt>
            <dd>{formatBytes(totalBytes)}</dd>
          </div>
          <div>
            <dt>Optimization flags</dt>
            <dd>{opportunityCount}</dd>
          </div>
          <div>
            <dt>Source path</dt>
            <dd>{scan.sourceFolder}</dd>
          </div>
        </div>
      ) : null}

      {result ? (
        <div className="scan-summary">
          <div>
            <dt>Staged output</dt>
            <dd>{result.stagingFolder}</dd>
          </div>
          <div>
            <dt>Before</dt>
            <dd>{formatBytes(sourceBytes)}</dd>
          </div>
          <div>
            <dt>After</dt>
            <dd>{formatBytes(outputBytes)}</dd>
          </div>
        </div>
      ) : null}

      {scan ? <ImageAssetTable assets={scan.assets} /> : null}
      {status ? <p className="tool-status">{status}</p> : null}
    </div>
  );
}

function ImageAssetTable({ assets }: { assets: ImageAssetEntry[] }) {
  return (
    <div className="asset-table-wrap">
      <table className="asset-table">
        <thead>
          <tr>
            <th>Asset</th>
            <th>Type</th>
            <th>Size</th>
            <th>Weight</th>
            <th>Opportunities</th>
          </tr>
        </thead>
        <tbody>
          {assets.map((asset) => (
            <tr key={asset.absolutePath}>
              <td>
                <strong>{asset.fileName}</strong>
                <span>{asset.relativePath}</span>
              </td>
              <td>{asset.inferredType}</td>
              <td>
                {asset.width}x{asset.height}
              </td>
              <td>{formatBytes(asset.fileSizeBytes)}</td>
              <td>{asset.opportunities.length > 0 ? asset.opportunities.join(", ") : "clean"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ModelOrientationTool({ project }: { project: ProjectSummary }) {
  const [catalog, setCatalog] = useState<ShipModelOrientationCatalog | null>(null);
  const [selectedPath, setSelectedPath] = useState("");
  const [draft, setDraft] = useState<ShipModelTransform>({
    yawDegrees: 0,
    pitchDegrees: 0,
    rollDegrees: 0,
    scale: 1
  });
  const [status, setStatus] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  async function loadCatalog() {
    setStatus("");
    const next = await projectApi.listShipModelOrientations(project.path);
    setCatalog(next);
    const first = next.models[0];
    if (first) {
      setSelectedPath((current) => current || first.modelPath);
      setDraft(first.transform);
    }
  }

  useEffect(() => {
    loadCatalog().catch((caught: unknown) =>
      setStatus(caught instanceof Error ? caught.message : String(caught))
    );
  }, [project.path]);

  const selected = catalog?.models.find((model) => model.modelPath === selectedPath) ?? null;

  function selectModel(model: ShipModelOrientationEntry) {
    setSelectedPath(model.modelPath);
    setDraft(model.transform);
  }

  function updateDraft(key: keyof ShipModelTransform, value: string) {
    const parsed = Number(value);
    setDraft((current) => ({
      ...current,
      [key]: Number.isFinite(parsed) ? parsed : current[key]
    }));
  }

  function rotateDraft(key: "yawDegrees" | "pitchDegrees" | "rollDegrees", delta: number) {
    setDraft((current) => ({
      ...current,
      [key]: current[key] + delta
    }));
  }

  async function saveDraft() {
    if (!selected) return;

    setIsSaving(true);
    setStatus("");
    try {
      const next = await projectApi.saveShipModelOrientation(
        project.path,
        selected.modelPath,
        draft
      );
      setCatalog(next);
      setStatus(`Saved ${selected.fileName}`);
    } catch (caught) {
      setStatus(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="orientation-tool">
      <div className="section-heading">
        <h3>Model Orientation</h3>
        <span>{catalog ? `${catalog.models.length} GLB models` : "loading"}</span>
      </div>

      <p className="tool-note">
        First slice: scan the linked game repo, save per-model yaw/pitch/roll/scale overrides,
        then apply that manifest in runtime or batch tooling.
      </p>

      <div className="orientation-layout">
        <div className="model-list">
          {catalog?.models.map((model) => (
            <button
              key={model.modelPath}
              type="button"
              className={model.modelPath === selectedPath ? "model-row active" : "model-row"}
              onClick={() => selectModel(model)}
            >
              <strong>{model.fileName}</strong>
              <span>{model.modelPath}</span>
            </button>
          ))}
          {catalog && catalog.models.length === 0 ? (
            <p className="empty-state">No GLB files found at assets/models/ships.</p>
          ) : null}
        </div>

        <section className="orientation-editor">
          {selected ? (
            <>
              <div>
                <p className="eyebrow">Selected Model</p>
                <h3>{selected.fileName}</h3>
                <p className="path-line">{selected.modelPath}</p>
              </div>

              <div className="orientation-controls">
                <label>
                  Yaw
                  <input
                    type="number"
                    step="15"
                    value={draft.yawDegrees}
                    onChange={(event) => updateDraft("yawDegrees", event.target.value)}
                  />
                </label>
                <label>
                  Pitch
                  <input
                    type="number"
                    step="15"
                    value={draft.pitchDegrees}
                    onChange={(event) => updateDraft("pitchDegrees", event.target.value)}
                  />
                </label>
                <label>
                  Roll
                  <input
                    type="number"
                    step="15"
                    value={draft.rollDegrees}
                    onChange={(event) => updateDraft("rollDegrees", event.target.value)}
                  />
                </label>
                <label>
                  Scale
                  <input
                    type="number"
                    step="0.05"
                    min="0.05"
                    value={draft.scale}
                    onChange={(event) => updateDraft("scale", event.target.value)}
                  />
                </label>
              </div>

              <div className="orientation-quick-controls" aria-label="Quick model rotation controls">
                <button type="button" onClick={() => rotateDraft("yawDegrees", -90)}>
                  Yaw -90
                </button>
                <button type="button" onClick={() => rotateDraft("yawDegrees", 90)}>
                  Yaw +90
                </button>
                <button type="button" onClick={() => rotateDraft("pitchDegrees", -90)}>
                  Pitch -90
                </button>
                <button type="button" onClick={() => rotateDraft("pitchDegrees", 90)}>
                  Pitch +90
                </button>
                <button type="button" onClick={() => rotateDraft("rollDegrees", -90)}>
                  Roll -90
                </button>
                <button type="button" onClick={() => rotateDraft("rollDegrees", 90)}>
                  Roll +90
                </button>
              </div>

              <ModelPreview
                projectPath={project.path}
                model={selected}
                transform={draft}
                onTransformChange={setDraft}
              />

              <div className="button-row">
                <button type="button" className="primary" disabled={isSaving} onClick={() => void saveDraft()}>
                  Save Orientation
                </button>
                <button
                  type="button"
                  disabled={isSaving}
                  onClick={() =>
                    setDraft({ yawDegrees: 0, pitchDegrees: 0, rollDegrees: 0, scale: 1 })
                  }
                >
                  Reset Draft
                </button>
              </div>

              <div className="manifest-readout">
                <dt>Manifest</dt>
                <dd>{catalog?.manifestPath ?? "Not loaded."}</dd>
              </div>
            </>
          ) : (
            <p className="empty-state">Select a model to edit orientation metadata.</p>
          )}
        </section>
      </div>

      {status ? <p className="tool-status">{status}</p> : null}
    </div>
  );
}

function ModelPreview({
  projectPath,
  model,
  transform,
  onTransformChange
}: {
  projectPath: string;
  model: ShipModelOrientationEntry;
  transform: ShipModelTransform;
  onTransformChange: (transform: ShipModelTransform) => void;
}) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const transformRef = useRef(transform);

  useEffect(() => {
    transformRef.current = transform;
  }, [transform]);

  useEffect(() => {
    const host = hostRef.current;
    if (!host || !isTauriRuntime()) return;
    const previewHost = host;

    const width = Math.max(360, previewHost.clientWidth);
    const height = Math.max(520, Math.min(700, Math.floor(window.innerHeight * 0.58)));
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x111817);

    const camera = new THREE.PerspectiveCamera(38, width / height, 0.01, 1000);
    const orbit = {
      yaw: THREE.MathUtils.degToRad(35),
      pitch: THREE.MathUtils.degToRad(24),
      distance: 6.6,
      target: new THREE.Vector3(0, 0, 0)
    };

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(width, height);
    previewHost.innerHTML = "";
    previewHost.appendChild(renderer.domElement);

    const group = new THREE.Group();
    scene.add(group);

    const ambient = new THREE.AmbientLight(0xffffff, 1.8);
    scene.add(ambient);
    const key = new THREE.DirectionalLight(0xffffff, 2.2);
    key.position.set(4, 5, 6);
    scene.add(key);
    const fill = new THREE.DirectionalLight(0x7fa8ff, 0.85);
    fill.position.set(-5, 2, -3);
    scene.add(fill);

    const grid = new THREE.GridHelper(8, 8, 0x55706a, 0x2f3e3a);
    scene.add(grid);

    const forwardArrow = new THREE.ArrowHelper(
      new THREE.Vector3(0, 0, -1),
      new THREE.Vector3(0, 0.08, 1.4),
      2.4,
      0xf2c65c,
      0.42,
      0.2
    );
    scene.add(forwardArrow);

    const upArrow = new THREE.ArrowHelper(
      new THREE.Vector3(0, 1, 0),
      new THREE.Vector3(1.25, 0, 0),
      1.8,
      0x78d9ff,
      0.34,
      0.16
    );
    scene.add(upArrow);

    let dragState:
      | {
          mode: "view" | "model-yaw";
          x: number;
          y: number;
          transform: ShipModelTransform;
        }
      | null = null;

    function updateCamera() {
      const clampedPitch = THREE.MathUtils.clamp(
        orbit.pitch,
        THREE.MathUtils.degToRad(-78),
        THREE.MathUtils.degToRad(78)
      );
      orbit.pitch = clampedPitch;
      camera.position.set(
        Math.sin(orbit.yaw) * Math.cos(clampedPitch) * orbit.distance,
        Math.sin(clampedPitch) * orbit.distance,
        Math.cos(orbit.yaw) * Math.cos(clampedPitch) * orbit.distance
      );
      camera.lookAt(orbit.target);
    }

    function handlePointerDown(event: PointerEvent) {
      event.preventDefault();
      previewHost.setPointerCapture(event.pointerId);
      dragState = {
        mode: event.shiftKey ? "model-yaw" : "view",
        x: event.clientX,
        y: event.clientY,
        transform: transformRef.current
      };
    }

    function handlePointerMove(event: PointerEvent) {
      if (!dragState) return;

      const dx = event.clientX - dragState.x;
      const dy = event.clientY - dragState.y;
      if (dragState.mode === "model-yaw") {
        onTransformChange({
          ...dragState.transform,
          yawDegrees: dragState.transform.yawDegrees + dx * 0.45,
          pitchDegrees: dragState.transform.pitchDegrees + dy * 0.45
        });
        return;
      }

      orbit.yaw -= dx * 0.008;
      orbit.pitch += dy * 0.008;
      dragState.x = event.clientX;
      dragState.y = event.clientY;
    }

    function handlePointerUp(event: PointerEvent) {
      dragState = null;
      if (previewHost.hasPointerCapture(event.pointerId)) {
        previewHost.releasePointerCapture(event.pointerId);
      }
    }

    function handleWheel(event: WheelEvent) {
      event.preventDefault();
      orbit.distance = THREE.MathUtils.clamp(orbit.distance + event.deltaY * 0.006, 2.2, 18);
    }

    previewHost.addEventListener("pointerdown", handlePointerDown);
    previewHost.addEventListener("pointermove", handlePointerMove);
    previewHost.addEventListener("pointerup", handlePointerUp);
    previewHost.addEventListener("pointercancel", handlePointerUp);
    previewHost.addEventListener("wheel", handleWheel, { passive: false });

    let disposed = false;
    let loadedRoot: THREE.Object3D | null = null;
    const loader = new GLTFLoader();
    void projectApi
      .loadShipModelPreview(projectPath, model.modelPath)
      .then((modelBytes) => {
        if (disposed) return;

        loader.parse(
          modelBytes,
          "",
          (gltf) => {
            if (disposed) return;

            loadedRoot = gltf.scene;
            group.add(loadedRoot);
            normalizeObjectToView(loadedRoot);
          },
          (error) => {
            if (!disposed) {
              previewHost.innerHTML = `<div class="preview-error">Could not parse model preview: ${String(error)}</div>`;
            }
          }
        );
      })
      .catch((error) => {
        if (!disposed) {
          previewHost.innerHTML = `<div class="preview-error">Could not load model preview: ${String(error)}</div>`;
        }
      });

    function applyDraftTransform() {
      const currentTransform = transformRef.current;
      group.rotation.set(
        THREE.MathUtils.degToRad(currentTransform.pitchDegrees),
        THREE.MathUtils.degToRad(currentTransform.yawDegrees),
        THREE.MathUtils.degToRad(currentTransform.rollDegrees)
      );
      group.scale.setScalar(Math.max(0.05, currentTransform.scale || 1));
    }

    function render() {
      if (disposed) return;
      applyDraftTransform();
      updateCamera();
      renderer.render(scene, camera);
      window.requestAnimationFrame(render);
    }

    render();

    return () => {
      disposed = true;
      if (loadedRoot) {
        loadedRoot.traverse((object) => {
          if (object instanceof THREE.Mesh) {
            object.geometry.dispose();
            const materials = Array.isArray(object.material) ? object.material : [object.material];
            for (const material of materials) {
              material.dispose();
            }
          }
        });
      }
      renderer.dispose();
      previewHost.removeEventListener("pointerdown", handlePointerDown);
      previewHost.removeEventListener("pointermove", handlePointerMove);
      previewHost.removeEventListener("pointerup", handlePointerUp);
      previewHost.removeEventListener("pointercancel", handlePointerUp);
      previewHost.removeEventListener("wheel", handleWheel);
      previewHost.innerHTML = "";
    };
  }, [projectPath, model.modelPath, onTransformChange]);

  return (
    <div className="model-preview-shell">
      <div className="section-heading">
        <h3>Preview</h3>
        <span>gold = expected forward, blue = expected up</span>
      </div>
      <p className="preview-help">Drag to orbit, wheel to zoom, shift-drag to rotate the model draft.</p>
      <div ref={hostRef} className="model-preview">
        {isTauriRuntime() ? null : (
          <p className="empty-state">3D preview is available in the desktop runtime.</p>
        )}
      </div>
    </div>
  );
}

function normalizeObjectToView(root: THREE.Object3D) {
  const bounds = new THREE.Box3().setFromObject(root);
  const size = new THREE.Vector3();
  const center = new THREE.Vector3();
  bounds.getSize(size);
  bounds.getCenter(center);

  root.position.sub(center);
  const maxDimension = Math.max(size.x, size.y, size.z);
  if (maxDimension > 0.001) {
    root.scale.setScalar(3.1 / maxDimension);
  }
}

createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <App />
  </StrictMode>
);
