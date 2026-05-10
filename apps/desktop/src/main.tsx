import { invoke } from "@tauri-apps/api/core";
import { FormEvent, StrictMode, useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

type ProjectSummary = {
  projectId: string;
  name: string;
  path: string;
  createdAt: string;
  updatedAt: string;
};

type ProjectFile = {
  schemaVersion: string;
  projectId: string;
  name: string;
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

type ActiveTool = "steam-launch-prep" | null;

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
    createdAt: now,
    updatedAt: now
  };
  const projectJson: ProjectFile = {
    schemaVersion: "0.1.0",
    projectId,
    name: cleanName,
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

function App() {
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [activeProject, setActiveProject] = useState<OpenProject | null>(null);
  const [activeTool, setActiveTool] = useState<ActiveTool>(null);
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [isBusy, setIsBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [projectName, setProjectName] = useState("");
  const [description, setDescription] = useState("");
  const [notes, setNotes] = useState("");

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
      setIsEditing(false);
      setDescription(opened.projectJson.description ?? "");
      setNotes(opened.projectJson.notes ?? "");
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
      setIsEditing(false);
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
          <button
            type="button"
            className="icon-button"
            aria-label="Add new project"
            title="Add new project"
            onClick={() => setIsCreateOpen(true)}
          >
            +
          </button>
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
                <p className="eyebrow">Open Project</p>
                <h2>{activeProject.project.name}</h2>
                <p className="path-line">{activeProject.project.path}</p>
              </div>
              <div className="project-actions">
                <button type="button" onClick={() => setIsEditing((value) => !value)}>
                  {isEditing ? "Cancel" : "Edit"}
                </button>
                <select
                  aria-label="Project tools"
                  value={activeTool ?? ""}
                  onChange={(event) =>
                    setActiveTool(
                      event.target.value === "steam-launch-prep" ? "steam-launch-prep" : null
                    )
                  }
                >
                  <option value="">Tools</option>
                  <option value="steam-launch-prep">Steam Launch Prep</option>
                </select>
              </div>
            </header>

            <div className="project-body">
              <section className="detail-card summary-card">
                <div className="section-heading">
                  <h3>Project Details</h3>
                  <span>{activeProject.projectJson.schemaVersion}</span>
                </div>

                {isEditing ? (
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
                        rows={8}
                        onChange={(event) => setNotes(event.target.value)}
                      />
                    </label>
                    <button type="submit" className="primary" disabled={isBusy}>
                      Save
                    </button>
                  </form>
                ) : (
                  <div className="readout-grid">
                    <div>
                      <dt>Description</dt>
                      <dd>{activeProject.projectJson.description || "No description yet."}</dd>
                    </div>
                    <div>
                      <dt>Notes</dt>
                      <dd>{activeProject.projectJson.notes || "No notes yet."}</dd>
                    </div>
                    <div>
                      <dt>Created</dt>
                      <dd>{formatDate(activeProject.project.createdAt)}</dd>
                    </div>
                    <div>
                      <dt>Updated</dt>
                      <dd>{formatDate(activeProject.project.updatedAt)}</dd>
                    </div>
                  </div>
                )}
              </section>

              <section className="detail-card tool-card">
                {activeTool === "steam-launch-prep" ? (
                  <SteamLaunchPrep project={activeProject.project} />
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

createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <App />
  </StrictMode>
);
