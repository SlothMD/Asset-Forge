# Dependency Policy

- Prefer dependencies that work offline once installed.
- Avoid dependencies that require hosted accounts for core local workflows.
- Keep external CLI tools optional and configurable per machine.
- Record required external tool assumptions in `refs/integrations`.
- Re-check binary and model optimizer compatibility against target runtimes before promotion.
