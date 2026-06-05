# Backup And Sync

Portable project folders should be safe to back up or sync through normal file sync or source control.

Do not sync:

- OS credential storage
- provider secrets
- machine-local executable paths as canonical project requirements
- app cache files that can be rebuilt

Do sync:

- project metadata
- source assets
- templates
- refs handoff manifests
- audit logs that are safe to share
