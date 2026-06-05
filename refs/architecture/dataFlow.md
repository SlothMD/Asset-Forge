# Data Flow

1. User opens or creates an Asset Forge project folder.
2. Portable project metadata is read from project files such as `project.json`.
3. Machine-local bindings map that project to local paths, tools, and external project links.
4. UI tools collect user edits, preview state, and validation results.
5. Tauri commands perform filesystem, Git, scan, conversion, optimization, and external-tool work.
6. Tool outputs are written to staging folders, exported folders, or project-local `refs/assetForge` manifests.
7. Handoff logs and manifests make outputs consumable by a developer, a coding agent, or a downstream game project.

Canonical data should remain inspectable. Cached indexes can exist later, but must be rebuildable from project files.
