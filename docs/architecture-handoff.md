# Architecture Handoff

## Architecture Thesis

Asset Foundry should be a local-first creator tool with a web-based interface, native desktop capabilities, deterministic local rendering, portable project files, and a future-compatible sync model.

The architecture should optimize for:

- Local executable distribution.
- Reuse of core UI and logic in a later SaaS portal.
- Portable, user-owned project data.
- Deterministic rendering and reproducible exports.
- Offline-capable core workflows.
- Native or sidecar workers for heavy local tasks.
- A small POC that does not block future SaaS support.

## Primary Stack

- Desktop shell: Tauri v2.
- Frontend: React plus TypeScript.
- Build tooling: Vite.
- Local database: SQLite for index/cache metadata.
- Project storage: human-readable project folders.
- Template model: declarative JSON schema.
- Rendering model: SVG-first, with raster and PDF export pipelines.
- Native backend: Rust through Tauri commands.
- Heavy workers: Rust first, sidecar workers only where useful.

## Core Boundaries

### UI Layer

The UI layer should not know whether a project is local or cloud-hosted. It should call shared services through narrow interfaces:

- `ProjectService`
- `DataSourceService`
- `TemplateService`
- `RenderService`
- `BuildService`
- `ExportService`
- `AssetService`

### Storage Layer

The storage layer should hide whether assets live in a local folder, SQLite cache, cloud object storage, or future synced storage.

V1 should implement:

- `LocalProjectStorageAdapter`
- `SQLiteMetadataAdapter`
- `LocalAssetStorageAdapter`

### Renderer Layer

The renderer must be deterministic. The same template, data row, style kit, asset references, and export preset should produce the same output.

Renderer inputs:

- Template JSON.
- Data row JSON.
- Style kit JSON.
- Asset references.
- Export preset.

Renderer outputs:

- Preview SVG or HTML.
- PNG image.
- PDF page or document.
- TTS deck or token sheet image.
- JSON manifest.

### Tauri Bridge Layer

Expose narrow commands to the frontend, such as:

- `open_project(path)`
- `save_project(project_id)`
- `import_csv(project_id, path)`
- `validate_project(project_id)`
- `render_component(project_id, component_id)`
- `render_build(project_id, build_id)`
- `export_build(project_id, build_id, export_preset_id)`
- `read_asset(asset_id)`
- `write_asset(asset_id, bytes)`

Avoid broad shell or file-system access from the frontend.

## Project Folder Model

```text
my-game-project/
  project.json
  data/
    cards.csv
    tokens.csv
    counters.csv
  assets/
    images/
    backgrounds/
    tokens/
    generated/
  icons/
    attack.svg
    move.svg
    supply.svg
  templates/
    poker-card-front.json
    poker-card-back.json
    circle-token.json
  style-kits/
    default.json
    sector-404.json
  builds/
    2026-05-10-playtest-01/
      build.json
      data-snapshot.json
      template-snapshot.json
      style-snapshot.json
      changelog.md
  exports/
    2026-05-10-playtest-01/
      png/
      pdf/
      tts/
      manifest.json
```

Project folders are inspectable, backup-friendly, source-control-friendly, and easier for creators to trust.

## POC Implementation Plan

1. Create Tauri v2, React, and TypeScript app shell.
2. Define `project.json`, template JSON, data source schema, and export preset schema.
3. Implement CSV import with header detection, row preview, and basic validation.
4. Render a hardcoded poker-card template using CSV rows.
5. Batch render rows to SVG or PNG and write a manifest.
6. Export a basic print-and-play PDF.
7. Export a TTS-style deck sheet PNG.
8. Persist project metadata and reopen the project folder.

## Technical Risks

- Tauri WebView differences: keep rendering simple in the POC and fall back to Electron only if needed.
- Preview/export drift: use a shared renderer package and template JSON as the source of truth.
- Print complexity: start with home print-and-play output.
- Template editor scope creep: start with JSON templates plus preview.
- SQLite lock-in: use SQLite only as an index/cache, not the canonical project format.
- AI distraction: keep AI out of the POC until the structured loop works.
