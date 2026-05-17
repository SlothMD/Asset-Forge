# Model Optimization Tool

Asset Forge includes a first-pass `Model Optimization` tool for staging runtime-ready GLB copies without modifying source assets.

Default Black Ledger Orbit small-craft paths:

```text
Source model folder: assets/models/small-craft
Staging folder: tmp/asset-forge-optimized-models
Manifest: refs/assetForge/model-optimization.manifest.json
```

The optimizer currently uses:

```text
npx @gltf-transform/cli optimize
```

Supported first-pass settings:

- geometry compression: Meshopt, Draco, Quantize, or none
- texture compression: WebP, KTX2, Auto, or none
- geometry simplification on/off
- target triangle count; simplify ratio is derived per model from `targetTriangles / currentTriangles`
- forced regeneration of existing staged outputs
- side-by-side review of original and optimized staged output

Every optimization run writes a manifest under `refs/assetForge` and appends a JSONL audit entry to:

```text
refs/assetForge/logs/asset-forge-audit.jsonl
```

Source GLB files are never overwritten by this tool.
