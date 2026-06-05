# Test Strategy

Asset Forge validation should combine fast code checks with workflow smoke tests.

Core layers:

- TypeScript typecheck for frontend/packages.
- Rust cargo checks for Tauri commands.
- Targeted smoke tests for project creation, path selection, scan, optimization, and manifest writing.
- Runtime compatibility checks for generated assets before promotion to a game project.

Long-running external tools should be validated with small fixture folders before full production batches.
