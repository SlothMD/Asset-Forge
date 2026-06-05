# Troubleshooting

## Tool Does Not Launch

- Verify dependencies are installed.
- Check the launcher script output.
- Run validation commands from `refs/testing/validationCommands.yaml`.

## Preview Cannot Load Asset

- Confirm the file path is accessible from the app.
- Confirm the asset format is supported by the previewer.
- For GLB files, check runtime compatibility flags such as meshopt and quantization.

## Batch Job Appears Stuck

- Check whether the tool has progress reporting.
- Check notification/audit logs.
- Prefer small fixture folders when validating new external tool integrations.
