# Current Handoff

## Current focus
Durable project memory has moved into the `refs/` structure. The latest work filled out project identity, module ownership, architecture boundaries, handoff/logging structures, validation commands, and testing criteria.

The newest implementation adds a generic Asset Handoff tool to the desktop UI. It initializes `refs/assetForge` in the linked game project, captures asset needs in `asset-needs.manifest.json`, attaches source assets into `asset-assignments.manifest.json`, optionally copies selected assets into the linked game folder, and regenerates `asset-handoff-to-game-agent.md` for coding-agent consumption.

## Relevant files
- `refs/project.yaml`
- `refs/architecture/boundaries.yaml`
- `refs/architecture/modules.yaml`
- `refs/planning/decisions.yaml`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/architecture/integrationMap.yaml`
- `refs/implementation/fileMap.yaml`
- `refs/testing/acceptanceCriteria.yaml`
- `refs/testing/validationCommands.yaml`
- `refs/handoffs/implementationLog.yaml`
- `docs/generic-asset-handoff.md`
- `docs/handoff-manifests.md`

## Decisions already made
- Asset Forge/Asset Foundry is local-first, with hosted account/project sync planned but not required for core workflows.
- `refs/architecture` owns durable architecture boundaries.
- `refs/planning` owns roadmap, todos, modules, decisions, and integration maps.
- `refs/handoffs` owns current handoff state and implementation logs.
- `refs/testing` owns durable acceptance criteria and validation commands.
- Portable project metadata and machine-local config/bindings must stay separate.
- Generated assets must stage first, then review/promote with manifests and audit logs.
- Generic game asset requests should flow through `refs/assetForge/asset-needs.manifest.json`; Asset Forge responses should flow back through `refs/assetForge/asset-assignments.manifest.json` and `asset-handoff-to-game-agent.md`.

## Known blockers
- `rust-msvc-sdk-linker`: previous notes say `cargo check` can be blocked on machines missing MSVC/Windows SDK `msvcrt.lib`.
- `cargo check` on 2026-06-04 still fails before app-crate checking with `LINK : fatal error LNK1104: cannot open file 'msvcrt.lib'`.
- Hosted identity, entitlement, and provider sync are planned but not implemented.
- Some integration config files are planned but do not exist yet under `refs/integrations`.

## Safe next steps
- Use `refs/project.yaml` and `refs/architecture/modules.yaml` before implementing new modules.
- Use `refs/architecture/boundaries.yaml` to check whether data belongs in portable project files, machine-local app data, or hosted services.
- Add missing acceptance criteria in `refs/testing/acceptanceCriteria.yaml` before starting larger workflow changes.
- Keep implementation logs in `refs/handoffs/implementationLog.yaml` when substantial changes land.

## Do not change
- Do not store credentials, provider tokens, Steam partner keys, GitHub tokens, or machine-only secrets in project files, refs, docs, examples, or tracked source.
- Do not overwrite source assets by default.
- Do not make hosted services required for core local workflows unless a feature is explicitly cloud-only.

## Validation commands
- `npm run typecheck`
- `npm run build`
- `cargo-fmt --check`
- `npm run tauri:build`
- `cargo check` from `apps/desktop/src-tauri` when the Rust/MSVC environment is healthy.

## Notes for next agent
- Documentation and project memory should now land in the appropriate `refs/` structure first; legacy docs can summarize but should not be the only source of durable planning state.
- If a referenced config file does not exist yet, mark it as planned instead of implying it is available.
