# Architecture Overview

Asset Forge is a local-first desktop workbench for repeatable game asset production.
The app uses a Tauri desktop shell, a React/TypeScript frontend, and Rust commands for filesystem, Git, external tool, and machine-local operations.

Portable project data should live in inspectable project folders. Machine-specific bindings, executable paths, local service URLs, and credentials should stay in app-local machine configuration or OS credential storage.

The core architecture separates:

- UI and interaction state in `apps/desktop/src`
- native capabilities in `apps/desktop/src-tauri`
- shared schemas and domain logic in `packages/core`
- storage adapters in `packages/storage`
- deterministic rendering/export helpers in `packages/renderer`
- worker boundaries in `packages/workers`
- durable project/agent memory in `refs`

Primary architecture constraints are documented in `refs/architecture/boundaries.yaml`.
