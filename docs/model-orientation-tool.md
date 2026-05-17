# Model Orientation Tool

Asset Forge includes a `Model Orientation` project tool for game repos that keep GLB assets in project-specific folders. The model folder is selectable in the tool; for Black Ledger Orbit ship models this is usually:

```text
assets/models/ships
```

The tool scans the selected folder inside the linked machine-local project folder and writes per-model transform overrides to a selectable handoff manifest. The default is:

```text
refs/assetForge/ship-model-orientation.manifest.json
```

Tool audit entries are also appended to:

```text
refs/assetForge/logs/asset-forge-audit.jsonl
```

This keeps the previewer and orientation editor generic while allowing each
project to bring its own folder layout. The `refs/assetForge` folder is intended
for coding-agent and downstream tooling handoff.

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
  },
  "audit": []
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
