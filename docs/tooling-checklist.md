# Asset Forge Tooling Checklist

Asset Forge should stay generic: project-specific folder conventions can be presets,
but the tools need selectable input/output paths and reusable manifests.

## Model Review And Hardpoints

- [x] First-pass model orientation manifest editor.
- [ ] Add configurable input model folder and output manifest path.
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

## Black Ledger Orbit Presets

- [ ] Ship models: scan `assets/models/ships`.
- [ ] Ship orientation output: `content/assets/ship-model-orientation.manifest.json`.
- [ ] Ship hardpoint output: `content/assets/ship-model-attachments.manifest.json`.
- [ ] Runtime textures: output optimized copies under `assets/textures`.
- [ ] PBR texture sets: prefer albedo, OpenGL normal, and roughness for the first runtime material pass.
