# Local Vs Hosted

Asset Forge defaults to local-first operation.

Local responsibilities:

- project creation and editing
- asset orientation, conversion, optimization, and review
- project-local manifests and audit logs
- Git operations through local credentials
- external tool execution on the current machine

Hosted responsibilities, later:

- account identity
- entitlement and license checks
- optional project-list sync
- provider integrations that require backend-only secrets
- hosted worker orchestration where local tools are unavailable

Portable project files must not require hosted state to remain usable.
