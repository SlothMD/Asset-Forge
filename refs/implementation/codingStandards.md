# Coding Standards

- Keep filesystem, shell, Git, and external-tool operations behind narrow Tauri commands.
- Prefer typed request/response structures at UI/backend boundaries.
- Keep UI components compact, predictable, and project-state oriented.
- Avoid storing absolute machine paths in portable project data.
- Add validation near user input and again at native command boundaries.
- Long-running jobs should expose progress, cancellation, or notification logs.
