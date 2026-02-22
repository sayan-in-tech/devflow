# Plugin Guide

`devflow` supports executable plugins and optional WASM plugin dispatch.

## Executable plugins

- Naming convention: `devflow-plugin-<name>`
- Discovery: `PATH` or `./plugins/`
- Protocol: JSON via stdin/stdout

Request shape:

```json
{ "command": "infra-check", "payload": {"key": "value"} }
```

Response shape:

```json
{ "ok": true, "message": "...", "data": {} }
```

Example plugin:

- `plugins/devflow-plugin-infra-check.py`

Run:

```bash
devflow plugin infra-check
```
