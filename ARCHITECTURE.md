# Architecture

> Technical architecture document for the `devflow` CLI.

This document describes the internal structure, data flow, module responsibilities, design patterns, and key decisions behind the `devflow` codebase. It is intended for contributors, maintainers, and engineers performing code review or knowledge transfer.

---

## Table of Contents

- [High-Level Overview](#high-level-overview)
- [System Diagram](#system-diagram)
- [Runtime Model](#runtime-model)
- [Module Map](#module-map)
- [Data Flow](#data-flow)
- [Key Design Decisions](#key-design-decisions)
- [Dependency Graph](#dependency-graph)
- [Error Handling Strategy](#error-handling-strategy)
- [Cross-Platform Considerations](#cross-platform-considerations)
- [Plugin Architecture](#plugin-architecture)
- [Security Model](#security-model)

---

## High-Level Overview

`devflow` is a single-binary Rust CLI built on:

- **clap v4** for argument parsing (derive-based)
- **tokio** for async runtime (used by file watcher, plugin subprocess I/O, port watching)
- **ratatui + crossterm** for the terminal dashboard UI
- **sysinfo** for process/CPU/memory inspection
- **notify v6** for filesystem change detection
- **serde + serde_yaml + serde_json** for config and data serialization
- **anyhow** for ergonomic error propagation
- **tracing + tracing-subscriber** for structured logging

The binary is entirely **local-first**: it makes no network calls, stores all state in the project directory (`.devflow/`), and operates deterministically. There is no daemon, no background service, and no server component.

---

## System Diagram

```
┌──────────────────────────────────────────────────────────┐
│                        User Shell                        │
│                  devflow <command> [args]                 │
└────────────────────────┬─────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│                      main.rs                             │
│  ┌────────────────┐  ┌────────────────┐                  │
│  │ tracing-subscriber │  │  Cli::parse()  │              │
│  │  (env filter)  │  │   (clap v4)    │                  │
│  └────────┬───────┘  └───────┬────────┘                  │
│           │                  │                           │
│           ▼                  ▼                           │
│        Logging         commands::run(cli)                │
└──────────────────────────┬───────────────────────────────┘
                           │
              ┌────────────┼────────────────┐
              ▼            ▼                ▼
        ┌─────────┐  ┌──────────┐   ┌────────────┐
        │commands/ │  │ plugin/  │   │  utils/    │
        │ up.rs    │  │ mod.rs   │   │ config.rs  │
        │ init.rs  │  │ (stdin/  │   │ envcheck.rs│
        │ env.rs   │  │  stdout  │   │ language.rs│
        │ port.rs  │  │  JSON)   │   │ ports.rs   │
        │ watch.rs │  └──────────┘   │ sanitize.rs│
        │ logs.rs  │                 │ snapshot.rs│
        │ deps.rs  │                 └────────────┘
        │ snap.rs  │
        │ dash.rs  │
        │ plugin.rs│
        └─────────┘
```

---

## Runtime Model

### Entry Point (`main.rs`)

1. Initialize `tracing-subscriber` with `EnvFilter` (respects `RUST_LOG` environment variable).
2. Parse CLI arguments via `Cli::parse()` (clap derive).
3. Dispatch to `commands::run(cli)` inside a `#[tokio::main]` async context.

### Async Model

The tokio runtime is used for:

- **`port --watch`**: `tokio::time::sleep` loop polling port ownership every 2 seconds.
- **`watch`**: `std::sync::mpsc` channel from `notify` watcher + `tokio::process::Command` for running tests.
- **`plugin`**: Spawning child processes with piped stdin/stdout via `tokio::process::Command`.
- **`dash`**: The TUI event loop uses `crossterm::event::poll` (blocking with timeout), not tokio events.

Most commands are effectively synchronous despite being `async fn` — the async boundary exists to keep the dispatch uniform and support the commands that genuinely need it.

### Process Lifecycle

```
CLI parse → command dispatch → execute → stdout output → exit
```

There is no long-running daemon. `devflow watch` and `devflow port --watch` run indefinitely until interrupted with `Ctrl+C`. `devflow dash` runs until the user presses `q`.

---

## Module Map

### `src/cli.rs` — CLI Definition

Defines the clap `Parser` struct and all subcommand/argument types:

| Type | Role |
|---|---|
| `Cli` | Root parser with `Command` subcommand |
| `Command` | Enum of all subcommands (`Up`, `Port`, `Watch`, etc.) |
| `PortArgs` | `--free`, `--watch`, `--port <N>` flags |
| `EnvArgs` | `doctor \| fix \| diff` mode enum |
| `SnapArgs` | `save \| restore` mode enum |
| `PluginArgs` | Plugin name + optional `--payload` JSON string |

### `src/commands/mod.rs` — Command Router

A single `pub async fn run(cli: Cli) -> Result<()>` that matches on `cli.command` and dispatches to the appropriate command module. This is the only function that crosses module boundaries for command execution.

### `src/commands/up.rs` — Environment Health Check

1. Detects project language via `language::detect_project_language`.
2. Checks whether the expected toolchain binary (e.g., `python`, `node`, `rustc`) is in `PATH` using `which::which`.
3. Reads the toolchain version hint file (`.nvmrc`, `rust-toolchain`, etc.).
4. Checks for `docker-compose.yml` / `compose.yaml`.
5. If `.devflow.yaml` exists, validates `.env` against the env schema.

### `src/commands/init.rs` — Config Generation

Writes a default `.devflow.yaml` with sample env schema, services, start commands, test command, ignore globs, and desired ports. Idempotent — skips if the file already exists.

### `src/commands/env.rs` — Environment Subcommands

Three modes:

- **`doctor`**: Aggregates `PATH` issues from `doctor_path_issues()` + env schema validation against `.env`.
- **`fix`**: Creates a minimal `.env` file if absent; no-op if it already exists.
- **`diff`**: Compares current `.env` with the snapshot at `.devflow/env_snapshot.json`. On first run, saves the initial snapshot.

### `src/commands/port.rs` — Port Diagnostics

- **`--free`**: Calls `common_free_ports()` which attempts `TcpListener::bind` on common ports (3000, 3001, 5173, 8000, 8080, 5432, 6379).
- **`--watch`**: Infinite loop polling `find_owner_by_port()` every 2s for ports 3000, 5173, 5432, 6379, 8080.
- **`--port <N>`**: One-shot process lookup + kill suggestions.

### `src/commands/watch.rs` — File Watcher

1. Loads ignore globs from `.devflow.yaml` (or defaults).
2. Creates a `notify::RecommendedWatcher` watching the project root recursively.
3. On filesystem events, filters out ignored paths using `globset`.
4. Runs `run_impacted_tests(language)` which spawns the appropriate test command (`pytest`, `jest`, `cargo test`, `go test`).

### `src/commands/logs.rs` — Log Analysis

1. Reads `devflow.log` from the project root.
2. Filters lines containing `ERROR` or `panic`.
3. Normalizes numeric tokens to `<n>` for grouping.
4. Compares against `.devflow/last_logs_state.json` to detect new errors since the last run.

### `src/commands/deps.rs` — Dependency Reports

Language-specific reports:

- **Python**: Checks for `requirements.txt` and `poetry.lock`.
- **Node**: Parses `package.json` to count declared dependencies.
- **Rust**: Checks for `Cargo.lock` and suggests `cargo tree` / `cargo deny`.

### `src/commands/snap.rs` — Process Snapshots

- **`save`**: Uses `sysinfo` to enumerate processes whose command line references the project directory. Captures env vars (excluding secrets). Writes to `.devflow/snapshot.json`.
- **`restore`**: Reads and prints the snapshot. Does not actually restart processes — restoration is advisory.

### `src/commands/dash.rs` — TUI Dashboard

Built with `ratatui` and `crossterm`:

1. Enters alternate screen, enables raw mode.
2. Renders a 3-panel layout: header, system health (CPU/MEM/PROCS), workspace info.
3. Refreshes every 400ms.
4. Exits on `q` keypress; restores terminal state.

### `src/commands/plugin.rs` — Plugin Dispatch Entry

Parses the `--payload` JSON (or defaults to `{}`) and calls `plugin::dispatch()`.

### `src/plugin/mod.rs` — Plugin Runtime

1. Resolves the plugin executable: tries `which devflow-plugin-<name>` first, then `./plugins/devflow-plugin-<name>`.
2. Spawns the process with piped stdin/stdout.
3. Writes a `PluginRequest` JSON to stdin.
4. Reads a `PluginResponse` JSON from stdout.
5. WASM plugins are recognized (`.wasm` suffix) but bail with a "not enabled" message.

### `src/utils/config.rs` — Configuration

Defines `DevflowConfig` and `ServiceDef` structs. Provides:

- `load_config(root)` → parse `.devflow.yaml` into `DevflowConfig`.
- `write_default_config(root)` → write a starter `.devflow.yaml`.

### `src/utils/envcheck.rs` — Environment Validation

- `parse_dotenv(root)` → read `.env` into `HashMap<String, String>`.
- `validate_env_schema(schema, actual)` → check each key exists and matches the declared type (`int`, `bool`, `string`).
- `doctor_path_issues()` → check `PATH` is set, Python and Node are available.

### `src/utils/language.rs` — Language Detection

Checks for marker files in priority order:

1. `pyproject.toml` or `requirements.txt` → Python
2. `package.json` → Node
3. `go.mod` → Go
4. `Cargo.toml` → Rust
5. Otherwise → Unknown

Also provides `expected_toolchain_hint()` which reads the first line of `.nvmrc`, `rust-toolchain`, `go.mod`, or `pyproject.toml`.

### `src/utils/ports.rs` — Port Utilities

- `common_free_ports()` → try-bind a list of common ports, return which succeed.
- `find_owner_by_port(port)` → scan all processes via `sysinfo`, match on port number appearing in process name or command.
- `safe_kill_suggestion(pid)` → return cross-platform kill command hints.
- `process_name(pid)` → look up a process name by PID.

### `src/utils/sanitize.rs` — Secret Redaction

Regex-based redaction of `password`, `token`, `secret`, and `apikey` values in both `key=value` and `key: "value"` formats. Used to prevent accidental credential leakage in logs and snapshots.

### `src/utils/snapshot.rs` — Snapshot Serialization

Defines `Snapshot` and `ProcSnapshot` structs. Captures:

- Timestamp (`chrono::Utc::now()`)
- Current working directory
- Filtered process list (processes referencing the project dir or named `cargo`)
- Environment variables (excluding `token` and `secret` keys)

---

## Data Flow

### `devflow up` Flow

```
current_dir
    │
    ├── detect_project_language(root)
    │       └── check marker files → Language enum
    │
    ├── which::which(toolchain_binary)
    │       └── PATH lookup → Ok(path) | Err
    │
    ├── expected_toolchain_hint(root)
    │       └── read .nvmrc / rust-toolchain → Option<String>
    │
    ├── check docker-compose.yml existence
    │
    └── if .devflow.yaml exists:
            ├── load_config(root) → DevflowConfig
            ├── parse_dotenv(root) → HashMap
            └── validate_env_schema(cfg.env, dotenv) → Vec<EnvIssue>
```

### `devflow watch` Flow

```
load_config → ignore_globs
detect_language → Language
    │
    ▼
notify::Watcher ──events──► mpsc::channel
                                │
                                ▼
                        filter ignored paths
                                │
                                ▼
                     run_impacted_tests(language)
                                │
                                ▼
                     tokio::process::Command
                     (pytest / jest / cargo test / go test)
```

### Plugin Dispatch Flow

```
CLI: devflow plugin <name> --payload '...'
    │
    ▼
PluginArgs { name, payload }
    │
    ▼
plugin::dispatch(name, payload)
    │
    ├── resolve_executable_plugin(name)
    │       ├── which("devflow-plugin-<name>")
    │       └── ./plugins/devflow-plugin-<name>
    │
    ├── spawn child process (stdin piped, stdout piped)
    │
    ├── write PluginRequest JSON to stdin
    │
    └── read PluginResponse JSON from stdout
```

---

## Key Design Decisions

### 1. Local-First, No Network

All operations run locally. No telemetry, no package registry calls, no remote APIs. This makes devflow fast, private, and usable in air-gapped environments.

### 2. Single Binary

No runtime dependencies, no separate daemon. The binary embeds everything needed. This simplifies installation and distribution.

### 3. Language Detection via Marker Files

Rather than requiring explicit configuration, devflow infers the project language by checking for well-known files (`Cargo.toml`, `package.json`, etc.). This enables zero-config usage.

### 4. Uniform Async Dispatch

All command handlers are `async fn` even when not strictly necessary. This avoids bifurcating the dispatch logic and makes it trivial to add async work to any command in the future.

### 5. Plugin Protocol: JSON over stdio

Plugins communicate via JSON on stdin/stdout. This is language-agnostic, debuggable, and requires no shared library or FFI. Plugins can be written in Python, Node.js, Go, Bash, or any language.

### 6. Snapshot = Advisory, Not Restorative

`devflow snap restore` prints what *would* be restored but does not automatically restart processes. This avoids dangerous side effects and respects the principle of least surprise.

### 7. Config File as Schema

`.devflow.yaml` doubles as both configuration and schema declaration. The `env` map defines expected types (`string`, `int`, `bool`), which are validated against the actual `.env` file.

### 8. Secret-Aware by Default

The `sanitize::redact` function and the snapshot env filter both strip sensitive values before any output. This is a defense-in-depth measure.

---

## Dependency Graph

### Runtime Dependencies

| Crate | Purpose | Version |
|---|---|---|
| `anyhow` | Error handling | 1.x |
| `chrono` | Timestamps (serde-enabled) | 0.4 |
| `clap` | CLI argument parsing (derive) | 4.5 |
| `crossterm` | Terminal control (raw mode, alternate screen, key events) | 0.28 |
| `globset` | Glob pattern matching for ignore rules | 0.4 |
| `ignore` | `.gitignore`-aware file traversal | 0.4 |
| `notify` | Cross-platform filesystem watcher | 6.x |
| `ratatui` | Terminal UI framework | 0.28 |
| `regex` | Secret redaction patterns | 1.x |
| `serde` | Serialization framework | 1.x |
| `serde_json` | JSON serialization | 1.x |
| `serde_yaml` | YAML config parsing | 0.9 |
| `sysinfo` | Process, CPU, memory inspection | 0.33 |
| `tokio` | Async runtime (full features) | 1.x |
| `tracing` | Structured logging | 0.1 |
| `tracing-subscriber` | Log output formatting | 0.3 |
| `walkdir` | Recursive directory traversal | 2.x |
| `which` | Executable path lookup | 7.x |

### Dev Dependencies

| Crate | Purpose |
|---|---|
| `assert_cmd` | CLI integration testing |
| `predicates` | Assertion matchers |
| `tempfile` | Temporary directories for tests |

---

## Error Handling Strategy

- All public functions return `anyhow::Result<()>`.
- Errors propagate via `?` to the top-level `main()`, which prints the error chain and exits with code 1.
- `context()` and `with_context()` are used to attach file paths and operation descriptions to errors.
- Panics are not used for control flow. The only panics in the codebase are in test assertions.

---

## Cross-Platform Considerations

| Concern | Approach |
|---|---|
| Path separators | Uses `std::path::Path` throughout; never hardcodes `/` or `\` |
| Process inspection | `sysinfo` crate abstracts OS-specific process APIs |
| File watching | `notify` crate uses `inotify` (Linux), `FSEvents` (macOS), `ReadDirectoryChangesW` (Windows) |
| Terminal UI | `crossterm` provides a unified API across all terminals |
| Kill suggestions | `safe_kill_suggestion()` outputs both Unix (`kill`) and Windows (`taskkill`) commands |
| CI matrix | GitHub Actions runs tests on `ubuntu-latest`, `macos-latest`, `windows-latest` |

---

## Plugin Architecture

### Discovery

```
1. Check $PATH for "devflow-plugin-<name>"
2. Check ./plugins/devflow-plugin-<name>
3. Fail with "plugin not found"
```

### Protocol

**Request** (JSON on stdin):
```json
{
  "command": "<plugin-name>",
  "payload": { ... }
}
```

**Response** (JSON on stdout):
```json
{
  "ok": true|false,
  "message": "human-readable status",
  "data": { ... }
}
```

### Lifecycle

1. devflow resolves the executable path.
2. Spawns a child process with piped stdin/stdout.
3. Writes the request JSON to stdin (async, in a spawned tokio task).
4. Waits for the child to exit.
5. If exit code != 0, returns an error.
6. Parses stdout as `PluginResponse`.
7. Pretty-prints the response to the user.

### WASM (Planned)

Plugin names ending in `.wasm` are recognized but currently bail with a "not enabled" error. The architecture is designed to support a WASM runtime (e.g., wasmtime) in the future.

---

## Security Model

| Risk | Mitigation |
|---|---|
| Secret leakage in output | `sanitize::redact()` strips password/token/secret/apikey patterns from all text |
| Secret leakage in snapshots | `save_snapshot()` filters out env vars containing "token" or "secret" |
| Malicious plugins | Plugins are external executables; users must trust what they install. devflow does not sandbox plugins |
| File system access | devflow only reads/writes within the project directory and `.devflow/` subdirectory |
| No network access | devflow makes zero network calls; no data exfiltration risk |
