# Asset Forge

Asset Forge is the working repository for Tabletop Asset Foundry, a local-first production tool suite for tabletop, board game, wargame, and strategy game creators.

The product thesis from the planning docs is direct: game data is the source of truth, templates turn that data into repeatable components, and exports are as important as editing. AI may assist with art and icons later, but the durable product is the structured asset pipeline.

## POC Goal

Build the smallest useful local-first proof of concept:

1. Launch a desktop app.
2. Create or open a local project folder.
3. Import a CSV with card data.
4. Preview rows and validation state.
5. Render cards from a hardcoded poker-card template.
6. Export individual PNG cards.
7. Export a basic print-and-play PDF.
8. Save project metadata.
9. Reopen the project and repeat the export.

## Initial Stack

- Desktop shell: Tauri v2
- Frontend: React, TypeScript, Vite
- Native backend: Rust through Tauri commands
- Project storage: inspectable local project folders
- Template model: declarative JSON
- Rendering model: SVG-first with PNG/PDF export paths
- Local index/cache: SQLite later, not as the only source of truth

## Repository Layout

```text
apps/
  desktop/        Tauri + React desktop app shell
packages/
  core/           Shared project schema, validation, and build logic
  renderer/       SVG-first preview and export rendering
  storage/        Local project storage adapters
  ui/             Shared editor and workflow components
  workers/        Export and image-processing worker boundaries
examples/
  sector-404/     Dogfood project fixtures
docs/
  product-brief.md
  architecture-handoff.md
```

## Development

Install Windows development dependencies from the repo root:

```powershell
.\install-dev-dependencies.cmd
```

The installer lists available drives with free and total capacity, then prompts for the drive to use for Rust/Cargo, npm global packages, npm cache, NuGet packages, pip cache, and user temp files. Run PowerShell as Administrator if the Visual Studio Build Tools install requests elevation.

Install dependencies from the repo root:

```powershell
npm install
```

Run the desktop frontend in browser mode:

```powershell
npm run dev
```

Build and launch the Tauri app after Rust/Cargo and Tauri prerequisites are installed:

```powershell
.\launch-asset-forge.cmd
```

Use `.\launch-asset-forge.cmd -BrowserOnly` to run the browser preview without the Tauri shell.
Use `.\launch-asset-forge.cmd -Dev` to run the faster Tauri dev shell.

Project metadata can store a portable GitHub repository URL. The local external project folder is stored per machine by the desktop app, so the same Asset Forge project can bind to different local clone paths on different computers before running Git status, pull, or push.

## Current Status

This is an initialized starter scaffold aligned with the product and architecture handoff. The next implementation step is wiring the project schema and CSV import flow.
