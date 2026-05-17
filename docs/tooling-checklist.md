# Asset Forge Tooling Checklist

Asset Forge should stay generic: project-specific folder conventions can be presets,
but the tools need selectable input/output paths and reusable manifests.

## Model Review And Hardpoints

- [x] First-pass model orientation manifest editor.
- [x] Add configurable input model folder and output manifest path.
- [ ] Add a model hardpoint editor for attachment metadata.
- [ ] Support hardpoint types:
  - `enginePort`
  - `maneuverThruster`
  - `weaponMountHint`
  - `cameraInspectAnchor`
  - `selectionAnchor`
- [ ] Show hardpoints in the previewer with editable position, rotation, radius, and label.
- [ ] Save hardpoints beside orientation transforms in a non-destructive asset manifest.
- [ ] Add optional Blender automation to bake approved transforms into generated copies.
- [ ] Keep original source models untouched unless the user explicitly promotes generated output.

## Generic Image Optimization

- [x] Add an `Image Optimization` tool with explicit source and staging folder fields.
- [x] Recursively scan source directory and subdirectories.
- [x] Classify files by type using a preset dropdown and filename heuristics:
  - albedo/base color
  - normal
  - roughness
  - metallic
  - ambient occlusion
  - height/displacement
  - UI/icon
  - screenshot/marketing
  - unknown
- [x] Show a readback table with dimensions, file size, format, inferred type, and optimization opportunities.
- [ ] Flag duplicate maps and missing paired maps.
- [x] Flag oversized textures, large PNGs, unknown types, and non-power-of-two dimensions.
- [ ] Let users choose target max size per type.
- [x] Let users choose a first-pass global target max size.
- [x] Let users choose output format.
- [ ] Support batch rename into project conventions.
- [x] Preserve source files and write optimized copies to a selected staging folder.
- [ ] Generate an optimization manifest that records source path, output path, dimensions, type, and transform decisions.
- [ ] Add presets for Godot runtime assets, web assets, marketing images, and UI icon sheets.
- [ ] Add progress reporting for scan and optimization runs.
- [ ] Dim and lock the tool screen while foreground processing is active.
- [ ] If optimization is moved to a background worker, add a notification log with start, progress, completion, and failure entries.
- [ ] Add cancellation support for long optimization runs.

## Generic Model Optimization

- [x] Add a `Model Optimization` tool with explicit source model folder, staging folder, and manifest path fields.
- [x] Recursively scan GLB files and report file size plus first-pass optimization opportunities.
- [x] Preserve source files and write optimized copies to a selected staging folder.
- [x] Use glTF Transform through `npx @gltf-transform/cli optimize` as the first optimizer backend.
- [x] Support geometry compression, texture compression, target triangle count, and forced regeneration.
- [x] Derive simplification ratio per model from target/current triangle count.
- [x] Add side-by-side original/optimized review from optimization output rows.
- [x] Write a model optimization manifest under `refs/assetForge` by default.
- [x] Append model optimization activity to `refs/assetForge/logs/asset-forge-audit.jsonl`.
- [ ] Add progress reporting and cancellation for long model optimization runs.
- [ ] Add optional configured executable support for local `gltfpack` or `gltf-transform` installs.
- [ ] Add inspection metrics for mesh count, material count, texture count, triangle count, and extension usage.
- [ ] Add promote/copy-to-runtime step after staged output is reviewed.

## Trellis/Comfy 3D Batch Conversion

- [x] Extract the folder-bound Trellis ship conversion script into a generic batch tool.
- [x] Support selectable source image folder, final output folder, workflow file, Comfy output folder, and oriented-input staging folder.
- [x] Support an optional source orientation manifest or CSV with rotate and flip instructions per source image.
- [x] Support output filename suffixes, batch limits, force regeneration, and dry-run checks.
- [x] Support first-pass mesh and texture settings through target face count and texture size parameters.
- [x] Add a first-pass UI wrapper for source/output selection, source scan, dry run, and conversion launch.
- [ ] Add progress reporting, cancellation, and a run log for long Comfy jobs.
- [ ] Add a contact-sheet/orientation review helper for source sprites before conversion.
- [ ] Add optional post-conversion model optimization and naming normalization.
- [ ] Add an import manifest writer for downstream projects such as Black Ledger Orbit.
- [ ] Add a Black Ledger Orbit small-craft preset for missiles, torpedoes, drones, fighters, and mines.

## Handoff Manifests And Audit Logs

- [x] Establish JSON as the first canonical handoff format for tool decisions.
- [x] Add source-image orientation manifest entries with file path, role, rotation, flips, notes, and update timestamp.
- [x] Add append-only audit records to source-image orientation manifests.
- [x] Make Trellis batch conversion consume the same source-image orientation manifest.
- [x] Keep source-image asset roles freeform and asset-specific.
- [x] Create project-local `refs/assetForge` and `refs/assetForge/logs` folders for handoff data.
- [x] Append project-local JSONL audit entries to `refs/assetForge/logs/asset-forge-audit.jsonl`.
- [ ] Add manifest export summaries for coding-agent handoff.
- [ ] Add manifest validation and stale-path warnings.
- [ ] Backfill existing image optimization output with an optimization manifest.
- [x] Backfill model orientation manifests with audit entries.

## This Computer Config

- [x] Add a top-level settings modal for machine-local configuration.
- [x] Show the current computer name.
- [x] Store generic tool entries with id, label, kind, executable path, URL, working directory, notes, and enabled state.
- [x] Store machine config as JSON under the Asset Forge app data folder for toolset and coding-agent consumption.
- [x] Add first-pass path picker controls for file and folder fields.
- [x] Path picker controls default to the current value or the last selected folder.
- [ ] Add tool health checks for URL services and local executables.
- [ ] Allow tools to read default paths from this config automatically.

## Black Ledger Orbit Presets

- [x] Ship models: scan a selected GLB folder, defaulting to `assets/models/ships`.
- [x] Ship orientation output: default to `refs/assetForge/ship-model-orientation.manifest.json`.
- [ ] Ship hardpoint output: `content/assets/ship-model-attachments.manifest.json`.
- [ ] Runtime textures: output optimized copies under `assets/textures`.
- [ ] PBR texture sets: prefer albedo, OpenGL normal, and roughness for the first runtime material pass.
- [ ] Small craft source images: scan `G:\My Drive\3D Files\source_images\Small Craft`.
- [ ] Small craft 3D output: write optimized runtime models into the Black Ledger Orbit asset tree.
