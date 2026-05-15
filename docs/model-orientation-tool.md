# Model Orientation Tool

Asset Forge now includes a first-pass `Model Orientation` project tool for game repos that keep GLB ship assets under:

```text
assets/models/ships
```

The tool scans the linked machine-local project folder and writes per-model transform overrides to:

```text
content/assets/ship-model-orientation.manifest.json
```

This is intentionally the immediate Black Ledger Orbit/Sector 404 path, not the
final generic Asset Forge contract. The reusable version should move these paths
into tool settings:

- input model folder
- output manifest folder/file
- optional generated-output folder for baked corrected models
- manifest schema/name preset

That keeps the previewer and orientation editor generic while allowing each
project to bring its own folder layout.

Current manifest shape:

```json
{
  "schemaVersion": "0.1.0",
  "updatedAt": "1760000000000Z",
  "models": {
    "res://assets/models/ships/example-token-optimized25k.glb": {
      "yawDegrees": 90,
      "pitchDegrees": 0,
      "rollDegrees": 0,
      "scale": 1
    }
  }
}
```

## Intended Black Ledger Orbit Use

The immediate use case is reviewing generated ship models that imported sideways or with inconsistent local forward axes. Runtime should read this manifest and apply the transform after loading the GLB, without destructively modifying source assets.

## Future Automation Path

Once the review loop is validated, Asset Forge can add a non-destructive and destructive workflow:

- Non-destructive: keep writing transform overrides and let the consuming game apply them at load time.
- Destructive/exported: use Blender automation to import each GLB, apply the approved transform, reset object transforms, and export a corrected GLB copy.

The preferred automated export path is a Blender Python script that:

1. Imports the source GLB.
2. Applies the manifest yaw/pitch/roll/scale to a root object.
3. Applies transforms to mesh data.
4. Exports a normalized GLB to a generated output folder.
5. Leaves the original GLB untouched unless explicitly promoted.

Do not auto-rotate generated assets purely from bounding-box heuristics without human confirmation. The safe workflow is detect likely-sideways models, present them in the review grid, and persist the chosen correction.
