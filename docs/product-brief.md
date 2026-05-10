# Product Brief

## Working Name

Asset Foundry, currently represented by this repository as Asset Forge.

## Thesis

Asset Foundry is a production workflow engine for tabletop and strategy game components. It turns structured game data into polished, repeatable, printable, and digital playtest-ready assets.

The product is not primarily an AI art generator. AI can assist with placeholder art, icons, emblems, textures, and variants, but the durable value is the repeatable pipeline:

```text
game data -> component templates -> batch render -> print and digital exports -> versioned playtest builds
```

## Core Promise

Design the component once. Update the data anytime. Regenerate everything.

## Target Users

- Solo and small-team board game designers prototyping from spreadsheets.
- Wargame and strategy game designers producing counters, faction sheets, scenario sheets, maps, and digital tabletop packages.
- Indie digital strategy developers needing 2D asset sheets, icons, counters, token states, UI badges, and data-linked sprite exports.

## Product Principles

- Data is the source of truth.
- Components must be printable and playable, not just attractive.
- Exports matter as much as editing.
- Board game and strategy workflows deserve first-class support.
- AI is an assistant, not the product.
- Every tool should reduce rework after playtest changes.
- The product should serve both physical and digital prototyping.
- Version history and reproducible builds are core features.
- The tool suite must be deployable as a local executable.
- SaaS deployment and context continuity should remain future-compatible.

## V1 Outcomes

- Create a local project per game.
- Manage data sources, templates, style kits, assets, export presets, and builds.
- Build reusable pipelines for cards, tokens, counters, and component sheets.
- Generate named playtest builds.
- Regenerate assets after data or template changes.
- Export print-focused and digital-tabletop-focused assets.
- Preserve project context locally while preparing for later SaaS sync.

## Early Dogfood Projects

- Sector 404 tactical space combat assets.
- A solar system 4X game sharing universe/style needs with Sector 404.
- A dungeon crawler with tiles, tokens, monsters, items, status markers, cards, and references.
- A family-weight pattern-matching board game with readable iconography and clean print-and-play output.

## Phase 0 Scope

- Local project file/folder.
- CSV import.
- Basic Card Forge.
- Basic Token Forge.
- Simple template rendering.
- Batch PNG export.
- Basic print sheet export.

## Non-Goals For The First POC

- Marketplace.
- SaaS collaboration.
- AI-first generation.
- Advanced 3D editing.
- Audio or video generation.
- Direct manufacturing automation.
- Full online playtesting platform.
