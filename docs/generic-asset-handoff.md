# Generic Asset Handoff Workflow

Asset Forge owns a generic handoff loop between a game coding agent and local asset tooling.

The workflow must not assume a specific game, engine, asset naming scheme, or runtime content format.

## Game Agent To Asset Forge

The game coding agent records needed assets in the linked game project:

```text
refs/
  assetForge/
    agent-asset-inventory-instructions.md
    asset-needs.manifest.json
```

Each asset need should state:

- stable id
- human label
- category such as token, card, model, texture, audio, UI, board, or background
- gameplay or UI purpose
- requested formats
- target runtime path if known
- priority and status
- implementation notes

## Asset Forge Work

Asset Forge tools may:

- create `refs/assetForge` when it is missing
- attach source files to asset needs
- copy included assets into the linked game folder when explicitly requested
- stage work under `refs/assetForge/staging`
- write audit entries to `refs/assetForge/logs/asset-forge-audit.jsonl`

## Asset Forge Back To Game Agent

Asset Forge writes:

```text
refs/
  assetForge/
    asset-assignments.manifest.json
    asset-handoff-to-game-agent.md
```

The game coding agent should read those files, then pull approved runtime assets into code/content using the project's own conventions.

The Markdown handoff is for quick human/agent review. The JSON manifests are canonical for programmatic consumption.
