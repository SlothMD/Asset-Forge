# Handoff Manifests

Asset Forge tools should write durable decisions as JSON manifests that are easy for the toolset, a game runtime, or a coding agent to consume. The format is intentionally explicit: paths, transforms, notes, and audit entries live beside the generated or source asset decision.

For project-specific decisions, tools should create this folder structure when it is missing:

```text
refs/
  assetForge/
    logs/
```

Canonical handoff manifests should live under `refs/assetForge`. Append-only tool activity should also be written as JSON Lines to:

```text
refs/assetForge/logs/asset-forge-audit.jsonl
```

## Source Image Orientation

The first source-image orientation manifest is used for 2D token orientation and Trellis/Comfy 3D conversion. Default project-local path:

```text
refs/assetForge/small-craft-source-orientation.manifest.json
```

```json
{
  "schemaVersion": "0.1.0",
  "updatedAt": "2026-05-16T12:00:00Z",
  "tool": "source-image-orientation",
  "sourceFolder": "G:\\My Drive\\3D Files\\source_images\\Small Craft",
  "assets": {
    "Drone_1.png": {
      "fileName": "Drone_1.png",
      "relativePath": "Drone_1.png",
      "absolutePath": "G:\\My Drive\\3D Files\\source_images\\Small Craft\\Drone_1.png",
      "transform": {
        "rotateDegrees": -45,
        "flipHorizontal": false,
        "flipVertical": false
      },
      "assetRole": "drone",
      "notes": "Source sprite corrected to south-facing token convention.",
      "updatedAt": "2026-05-16T12:00:00Z"
    }
  },
  "audit": [
    {
      "timestamp": "2026-05-16T12:00:00Z",
      "tool": "source-image-orientation",
      "action": "saved-orientation",
      "target": "Drone_1.png",
      "summary": "Updated source image orientation metadata for downstream tools and game integration."
    }
  ]
}
```

Downstream consumers should key by `relativePath` when possible. `absolutePath` is included as a convenience for local tool runs, but should not be treated as portable.

## Machine Config

Machine-local tool locations are stored outside project repos in the Asset Forge app data folder as `machine-config.json`. This is intentionally writable by tools or coding agents when they discover installed software.

Each tool entry includes:

- `id`: stable identifier such as `comfyui`, `blender`, or `imagemagick`
- `kind`: generic integration type such as `cli`, `desktop-app`, or `http-service`
- `executablePath`, `url`, and `workingDirectory`: optional reachability fields
- `notes`: human-readable operational notes
- `enabled`: whether Asset Forge should consider the tool available on this machine
