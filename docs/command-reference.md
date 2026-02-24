# Command Reference

> Quick reference for all devflow CLI commands, flags, and options.

---

## Global Options

```
devflow --version       Print version
devflow --help          Print help
devflow <command> -h    Print help for a specific command
```

---

## Commands

### `devflow up`

Check local development environment health.

```bash
devflow up
```

**Detects**: project language, toolchain availability, docker-compose files, env schema compliance.
**Requires**: none (`.devflow.yaml` optional but recommended).
**Output**: Multi-line status report to stdout.

---

### `devflow init`

Generate a default `.devflow.yaml` configuration file.

```bash
devflow init
```

**Idempotent**: Will not overwrite an existing file.
**Side effects**: Creates `.devflow.yaml` in the current directory.

---

### `devflow env`

Environment diagnostics and management.

```bash
devflow env doctor      # Diagnose PATH, toolchain, and env schema issues
devflow env fix         # Create a minimal .env file if absent
devflow env diff        # Compare current .env to saved snapshot
```

| Subcommand | Description | Side Effects |
|---|---|---|
| `doctor` | Aggregates PATH, Python, Node availability checks + env schema validation | None |
| `fix` | Creates `.env` with comment header | Creates `.env` if missing |
| `diff` | Shows added/changed/removed keys vs. baseline | Creates `.devflow/env_snapshot.json` on first run |

---

### `devflow port`

Port diagnostics and monitoring.

```bash
devflow port                    # Inspect default port 3000
devflow port --port 8080        # Inspect a specific port
devflow port --free             # List free common ports (JSON)
devflow port --watch            # Live-monitor ports every 2s
```

| Flag | Short | Type | Default | Description |
|---|---|---|---|---|
| `--port` | `-p` | `u16` | `3000` | Port number to inspect |
| `--free` | | bool | `false` | Print free common ports as JSON array |
| `--watch` | | bool | `false` | Continuously monitor ports (Ctrl+C to stop) |

**Monitored ports** (in `--watch` mode): 3000, 5173, 5432, 6379, 8080.

**Free port check list**: 3000, 3001, 5173, 8000, 8080, 5432, 6379.

---

### `devflow watch`

Watch filesystem for changes and automatically run tests.

```bash
devflow watch
```

**Behavior**: Recursively watches the project directory. On file changes (excluding ignored paths), runs the language-appropriate test command.

**Runs indefinitely** — press `Ctrl+C` to stop.

| Language | Test Command |
|---|---|
| Python | `pytest -q` |
| Node | `npx jest --passWithNoTests` |
| Rust | `cargo test` |
| Go | `go test ./...` |

**Configuration**: Set `ignore_globs` in `.devflow.yaml` to control which files are ignored.

---

### `devflow logs`

Analyze error logs and track new errors.

```bash
devflow logs
```

**Input**: Reads `devflow.log` from the current directory.
**Output**: Grouped error frequencies and newly-seen errors since last run.
**State**: `.devflow/last_logs_state.json`

---

### `devflow deps`

Print dependency metadata for the detected project type.

```bash
devflow deps
```

| Language | Analysis |
|---|---|
| Python | Checks `requirements.txt`, `poetry.lock` |
| Node | Counts packages in `package.json` |
| Rust | Checks `Cargo.lock`, suggests `cargo tree` / `cargo deny` |
| Go | Not yet implemented |

---

### `devflow snap`

Save and inspect development workspace snapshots.

```bash
devflow snap save       # Capture current state
devflow snap restore    # View saved snapshot
```

| Subcommand | Description | Side Effects |
|---|---|---|
| `save` | Captures processes, env vars, timestamp | Writes `.devflow/snapshot.json` |
| `restore` | Prints snapshot contents | None (advisory only) |

**Security**: Env vars containing "token" or "secret" are excluded from snapshots.

---

### `devflow dash`

Open an interactive TUI dashboard.

```bash
devflow dash
```

**Panels**: Header, System health (CPU/MEM/PROCS), Workspace info.
**Refresh**: Every 400ms.
**Exit**: Press `q`.

---

### `devflow plugin`

Run an external executable plugin.

```bash
devflow plugin <name>                          # Run with empty payload
devflow plugin <name> --payload '{"key":"v"}'  # Run with JSON payload
```

| Argument | Type | Required | Description |
|---|---|---|---|
| `<name>` | string | Yes | Plugin name (with or without `devflow-plugin-` prefix) |
| `--payload` | string | No | JSON string to pass to the plugin |

**Plugin resolution**: `PATH` lookup first, then `./plugins/` directory.
**Protocol**: JSON via stdin/stdout. See [plugin.md](plugin.md) for details.
