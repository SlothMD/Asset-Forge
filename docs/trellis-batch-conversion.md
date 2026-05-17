# Trellis/Comfy Batch Conversion

`tools/Convert-Trellis2Batch.ps1` converts a folder of PNG source images into GLB models through a ComfyUI Trellis workflow. It is the generic replacement for the earlier folder-bound ship conversion script.

## Dry Run

Use `-WhatIfOnly` first to confirm source enumeration, output names, and orientation records.

```powershell
.\tools\Convert-Trellis2Batch.ps1 `
  -SourceDir "G:\My Drive\3D Files\source_images\Small Craft" `
  -FinalOutputDir "D:\Apps\Black-Ledger-Orbit\assets\models\small-craft" `
  -OrientationCsv "D:\Apps\Black-Ledger-Orbit\refs\assetForge\small-craft-source-orientation.manifest.json" `
  -OutputSuffix "raw" `
  -TargetFaceCount 500000 `
  -TextureSize 2048 `
  -WhatIfOnly
```

## Real Conversion

ComfyUI must be running and reachable through `-ComfyUrl`. The default is `http://127.0.0.1:8000`.

```powershell
.\tools\Convert-Trellis2Batch.ps1 `
  -SourceDir "G:\My Drive\3D Files\source_images\Small Craft" `
  -FinalOutputDir "D:\Apps\Black-Ledger-Orbit\assets\models\small-craft" `
  -OrientationCsv "D:\Apps\Black-Ledger-Orbit\refs\assetForge\small-craft-source-orientation.manifest.json" `
  -OutputSuffix "raw" `
  -TargetFaceCount 500000 `
  -TextureSize 2048
```

The script preserves source images. Any rotated or flipped inputs are written to the staging folder before upload.

## Orientation Manifest

The orientation manifest is optional. If a file has no entry, it is uploaded as-is. CSV is still supported for simple batch jobs, but JSON is the canonical handoff format because it also carries role, notes, and audit history.

```json
{
  "assets": {
    "Drone_1.png": {
      "fileName": "Drone_1.png",
      "relativePath": "Drone_1.png",
      "transform": {
        "rotateDegrees": -45,
        "flipHorizontal": false,
        "flipVertical": false
      }
    }
  }
}
```

`rotateDegrees` supports arbitrary degree values. Right-angle rotations use .NET `RotateFlipType`; diagonal craft use a transparent redraw pass so source sprites can be normalized before upload.

## Current Black Ledger Orbit Notes

The mines are already south-facing. The drones, fighters, missiles, and torpedoes need orientation review before final conversion, because their source art does not consistently match the ship-token south-facing convention.
