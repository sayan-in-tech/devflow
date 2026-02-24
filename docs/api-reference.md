# API Reference — Internal Module Documentation

> This document provides a complete reference of every public type, function, and module in the `devflow` codebase. It is intended for contributors and engineers performing code-level work.

---

## Table of Contents

- [Crate Structure](#crate-structure)
- [Module: `cli`](#module-cli)
- [Module: `commands`](#module-commands)
  - [`commands::up`](#commandsup)
  - [`commands::init`](#commandsinit)
  - [`commands::env`](#commandsenv)
  - [`commands::port`](#commandsport)
  - [`commands::watch`](#commandswatch)
  - [`commands::logs`](#commandslogs)
  - [`commands::deps`](#commandsdeps)
  - [`commands::snap`](#commandssnap)
  - [`commands::dash`](#commandsdash)
  - [`commands::plugin`](#commandsplugin)
- [Module: `plugin`](#module-plugin)
- [Module: `utils`](#module-utils)
  - [`utils::config`](#utilsconfig)
  - [`utils::envcheck`](#utilsenvcheck)
  - [`utils::language`](#utilslanguage)
  - [`utils::ports`](#utilsports)
  - [`utils::sanitize`](#utilssanitize)
  - [`utils::snapshot`](#utilssnapshot)

---

## Crate Structure

```
devflow (lib)
├── cli          — CLI argument definitions (clap derive)
├── commands     — Subcommand implementations
│   ├── up       — Environment health check
│   ├── init     — Config file generation
│   ├── env      — Environment doctor/fix/diff
│   ├── port     — Port diagnostics
│   ├── watch    — File watcher + test runner
│   ├── logs     — Log analysis
│   ├── deps     — Dependency reports
│   ├── snap     — Process snapshots
│   ├── dash     — TUI dashboard
│   └── plugin   — Plugin dispatch entry
├── plugin       — Plugin resolution and execution runtime
└── utils        — Shared utility functions
    ├── config   — YAML config parsing
    ├── envcheck — Dotenv parsing and schema validation
    ├── language — Project language detection
    ├── ports    — Port scanning and process lookup
    ├── sanitize — Secret redaction
    └── snapshot — Process/env snapshot serialization
```

---

## Module: `cli`

**File**: `src/cli.rs`

Defines the clap-based CLI interface using the derive API.

### Structs

#### `Cli`

```rust
#[derive(Debug, Parser)]
#[command(name = "devflow", version, about = "Developer workflow automation")]
pub struct Cli {
    pub command: Command,
}
```

Root CLI parser. Contains a single field `command` which is the selected subcommand.

#### `PortArgs`

```rust
pub struct PortArgs {
    pub free: bool,     // --free: list free ports
    pub watch: bool,    // --watch: live monitor
    pub port: Option<u16>,  // --port <N>: inspect specific port
}
```

#### `EnvArgs`

```rust
pub struct EnvArgs {
    pub mode: EnvMode,  // doctor | fix | diff
}
```

#### `SnapArgs`

```rust
pub struct SnapArgs {
    pub mode: SnapMode,  // save | restore
}
```

#### `PluginArgs`

```rust
pub struct PluginArgs {
    pub name: String,             // positional: plugin name
    pub payload: Option<String>,  // --payload: optional JSON string
}
```

### Enums

#### `Command`

```rust
pub enum Command {
    Up,
    Port(PortArgs),
    Watch,
    Env(EnvArgs),
    Logs,
    Deps,
    Snap(SnapArgs),
    Dash,
    Init,
    Plugin(PluginArgs),
}
```

#### `EnvMode`

```rust
pub enum EnvMode { Doctor, Fix, Diff }
```

#### `SnapMode`

```rust
pub enum SnapMode { Save, Restore }
```

---

## Module: `commands`

**File**: `src/commands/mod.rs`

### Functions

#### `run(cli: Cli) -> Result<()>`

Main command dispatcher. Matches on `cli.command` and routes to the appropriate handler module. This is the single entry point called from `main()`.

**Parameters**:
- `cli: Cli` — Parsed CLI arguments.

**Returns**: `anyhow::Result<()>`

**Behavior**: Dispatches to the corresponding `async fn run()` in each command module. For `Env` and `Snap`, further dispatches by mode.

---

### `commands::up`

**File**: `src/commands/up.rs`

#### `run() -> Result<()>`

Performs a comprehensive environment health check:

1. Gets the current working directory.
2. Detects project language via `detect_project_language()`.
3. Checks for the expected toolchain binary in `PATH` (maps: Python→`python`, Node→`node`, Go→`go`, Rust→`rustc`).
4. Reads the toolchain version hint from files like `.nvmrc`, `rust-toolchain`.
5. Checks for Docker Compose files (`docker-compose.yml` or `compose.yaml`).
6. If `.devflow.yaml` exists, loads the config and validates `.env` against the env schema.

**Output**: Prints a multi-line status report to stdout.

---

### `commands::init`

**File**: `src/commands/init.rs`

#### `run() -> Result<()>`

Creates a default `.devflow.yaml` in the current directory. Idempotent — if the file already exists, prints a message and returns without modification.

**Side effects**: Writes `.devflow.yaml` to disk.

---

### `commands::env`

**File**: `src/commands/env.rs`

#### `doctor() -> Result<()>`

Aggregates environment health issues:

1. Calls `doctor_path_issues()` to check `PATH`, Python, and Node availability.
2. If `.devflow.yaml` exists, validates `.env` against the declared env schema.
3. Prints all issues or "healthy" if none found.

#### `fix() -> Result<()>`

Creates a minimal `.env` file with a comment header if one doesn't exist. No-op if `.env` already exists.

#### `diff() -> Result<()>`

Compares the current `.env` with a previously saved snapshot (`.devflow/env_snapshot.json`):

- **First run**: Saves the current env as the baseline snapshot.
- **Subsequent runs**: Prints added, changed, and removed keys.

**State files**: `.devflow/env_snapshot.json`

---

### `commands::port`

**File**: `src/commands/port.rs`

#### `run(args: PortArgs) -> Result<()>`

Port inspection and monitoring:

- **`--free`**: Calls `common_free_ports()` and prints the result as a JSON array.
- **`--watch`**: Enters an infinite loop, polling ports 3000, 5173, 5432, 6379, 8080 every 2 seconds and printing owner info.
- **`--port <N>` (or default 3000)**: One-shot lookup of the process owning the specified port, plus cross-platform kill suggestions.

---

### `commands::watch`

**File**: `src/commands/watch.rs`

#### `run() -> Result<()>`

File watcher with automatic test execution:

1. Detects project language.
2. Loads ignore globs from `.devflow.yaml` (defaults to empty).
3. Creates a `notify::RecommendedWatcher` on the project root (recursive).
4. On each batch of filesystem events, filters ignored paths and runs language-specific tests.

**Test commands by language**:
| Language | Command |
|---|---|
| Python | `pytest -q` |
| Node | `npx jest --passWithNoTests` |
| Rust | `cargo test` |
| Go | `go test ./...` |
| Unknown | (skipped) |

**Runs indefinitely** until interrupted with `Ctrl+C`.

---

### `commands::logs`

**File**: `src/commands/logs.rs`

#### `run() -> Result<()>`

Log analysis and error tracking:

1. Reads `devflow.log` from the project root.
2. Filters for lines containing `ERROR` or `panic`.
3. Normalizes numeric tokens to `<n>` (via `normalize_trace()`).
4. Groups identical normalized traces and counts frequency.
5. Compares with the previous run's state (`.devflow/last_logs_state.json`) to identify new errors.
6. Saves the current state for the next run.

**State files**: `.devflow/last_logs_state.json`

#### `normalize_trace(line: &str) -> String` (private)

Replaces purely numeric whitespace-separated tokens with `<n>` to deduplicate error messages that differ only in PIDs, line numbers, etc.

---

### `commands::deps`

**File**: `src/commands/deps.rs`

#### `run() -> Result<()>`

Prints dependency metadata based on detected project language:

- **Python**: Checks `requirements.txt` and `poetry.lock` existence.
- **Node**: Parses `package.json` to count declared dependencies.
- **Rust**: Checks `Cargo.lock` existence; suggests `cargo tree` and `cargo deny`.
- **Go / Unknown**: Prints "not yet available".

---

### `commands::snap`

**File**: `src/commands/snap.rs`

#### `save() -> Result<()>`

Delegates to `snapshot::save_snapshot()`. Prints confirmation message.

#### `restore() -> Result<()>`

Delegates to `snapshot::read_snapshot()`. Prints the saved timestamp, working directory, and each captured process name and command. Does **not** restart processes.

---

### `commands::dash`

**File**: `src/commands/dash.rs`

#### `run() -> Result<()>`

Launches an interactive TUI dashboard:

1. Enables raw mode, switches to alternate screen.
2. Creates a 3-panel vertical layout using `ratatui`:
   - **Header**: "devflow dash (press q to quit)"
   - **System**: CPU usage %, used memory KB, process count
   - **Workspace**: Static hints about other devflow commands
3. Refreshes system metrics every 400ms via `sysinfo`.
4. Exits on `q` keypress.
5. Restores terminal to normal mode.

---

### `commands::plugin`

**File**: `src/commands/plugin.rs`

#### `run(args: PluginArgs) -> Result<()>`

Entry point for plugin execution:

1. Parses `--payload` JSON string (or defaults to `{}`). If parsing fails, wraps raw string in `{"raw": "..."}`.
2. Calls `plugin::dispatch(name, payload)`.
3. Pretty-prints the `PluginResponse`.

---

## Module: `plugin`

**File**: `src/plugin/mod.rs`

### Types

#### `PluginRequest`

```rust
pub struct PluginRequest {
    pub command: String,
    pub payload: serde_json::Value,
}
```

Serialized to JSON and written to the plugin's stdin.

#### `PluginResponse`

```rust
pub struct PluginResponse {
    pub ok: bool,
    pub message: String,
    pub data: serde_json::Value,
}
```

Deserialized from the plugin's stdout.

### Functions

#### `dispatch(name: &str, payload: Value) -> Result<PluginResponse>`

Main plugin execution function:

1. If `name` ends with `.wasm`, bails with "WASM plugin runtime not enabled".
2. Resolves the executable path via `resolve_executable_plugin()`.
3. Spawns the child process with piped stdin/stdout.
4. Writes `PluginRequest` to stdin asynchronously.
5. Waits for exit; returns error if non-zero exit code.
6. Parses stdout as `PluginResponse`.

#### `resolve_executable_plugin(name: &str) -> Result<PathBuf>` (private)

Resolution order:
1. Prepend `devflow-plugin-` if not already present.
2. `which::which(prefixed_name)` — searches `PATH`.
3. `./plugins/<prefixed_name>` — local project plugins directory.
4. Fails with "plugin not found".

---

## Module: `utils`

**File**: `src/utils/mod.rs`

Re-exports all utility submodules:
```rust
pub mod config;
pub mod envcheck;
pub mod language;
pub mod ports;
pub mod sanitize;
pub mod snapshot;
```

---

### `utils::config`

**File**: `src/utils/config.rs`

#### Types

##### `DevflowConfig`

```rust
pub struct DevflowConfig {
    pub env: HashMap<String, String>,       // env key → expected type (string/int/bool)
    pub services: Vec<ServiceDef>,          // named service definitions
    pub start_commands: Vec<String>,        // shell commands to run on startup
    pub test_command: Option<String>,       // test runner command
    pub ignore_globs: Vec<String>,          // glob patterns for watch ignore
    pub desired_ports: Vec<u16>,            // ports this project uses
}
```

All fields have `#[serde(default)]` and the struct derives `Default`.

##### `ServiceDef`

```rust
pub struct ServiceDef {
    pub name: String,
    pub command: String,
}
```

#### Functions

##### `load_config(root: &Path) -> Result<DevflowConfig>`

Reads and parses `.devflow.yaml` from the given root directory.

**Errors**: File not found or YAML parse errors (with context attached).

##### `write_default_config(root: &Path) -> Result<()>`

Writes a default `.devflow.yaml` with sample values:
- `env`: `DATABASE_URL: string`, `PORT: int`
- `services`: one service named "app" with `cargo run`
- `start_commands`: `docker compose up -d`
- `test_command`: `cargo test`
- `ignore_globs`: `target/**`, `node_modules/**`
- `desired_ports`: 3000, 5432

---

### `utils::envcheck`

**File**: `src/utils/envcheck.rs`

#### Types

##### `EnvIssue`

```rust
pub struct EnvIssue {
    pub key: String,
    pub reason: String,
}
```

#### Functions

##### `parse_dotenv(root: &Path) -> Result<HashMap<String, String>>`

Parses a `.env` file from the root directory into a key-value map.

**Behavior**:
- Skips blank lines, comment lines (`#`), and lines without `=`.
- Splits on the first `=` only.
- Trims whitespace from keys and values.
- Returns an empty map if `.env` doesn't exist.

##### `validate_env_schema(schema: &HashMap<String, String>, actual: &HashMap<String, String>) -> Vec<EnvIssue>`

Validates actual env vars against the declared schema:

| Schema Type | Validation |
|---|---|
| `string` | Only checks key presence |
| `int` | Key must exist and value must parse as `i64` |
| `bool` | Key must exist and value must parse as `bool` |

Returns a `Vec<EnvIssue>` with one entry per problem.

##### `doctor_path_issues() -> Vec<String>`

System-level diagnostics:
1. Checks if `PATH` environment variable is set.
2. Checks if `python` or `python3` is in `PATH`.
3. Checks if `node` is in `PATH`.

Returns a list of human-readable issue strings.

---

### `utils::language`

**File**: `src/utils/language.rs`

#### Types

##### `Language`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Python,
    Node,
    Go,
    Rust,
    Unknown,
}
```

#### Functions

##### `detect_project_language(root: &Path) -> Language`

Detects project language by checking for marker files in priority order:

| Priority | File(s) | Language |
|---|---|---|
| 1 | `pyproject.toml` or `requirements.txt` | Python |
| 2 | `package.json` | Node |
| 3 | `go.mod` | Go |
| 4 | `Cargo.toml` | Rust |
| 5 | (none) | Unknown |

**Note**: First match wins. A project with both `pyproject.toml` and `Cargo.toml` is detected as Python.

##### `expected_toolchain_hint(root: &Path) -> Option<String>`

Reads the first line of the first matching version hint file:
- `.nvmrc`
- `rust-toolchain`
- `go.mod`
- `pyproject.toml`

Returns `None` if no hint files exist.

---

### `utils::ports`

**File**: `src/utils/ports.rs`

#### Types

##### `PortOwner`

```rust
pub struct PortOwner {
    pub port: u16,
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub cmd: String,
    pub memory_kb: u64,
    pub uptime_secs: u64,
}
```

#### Functions

##### `common_free_ports() -> Vec<u16>`

Attempts to bind a TCP listener on `127.0.0.1` for each of these ports: 3000, 3001, 5173, 8000, 8080, 5432, 6379. Returns the ports where binding succeeded (i.e., the port is free).

##### `find_owner_by_port(port: u16) -> Option<PortOwner>`

Scans all system processes via `sysinfo` and returns the first process whose name or command line arguments contain the port number as a substring.

**Caveat**: This is a heuristic — it matches on string containment, not actual socket inspection. A process with PID 3000 could false-positive for port 3000.

##### `safe_kill_suggestion(pid: u32) -> Vec<String>`

Returns cross-platform kill command suggestions for the given PID:
- `kill <pid>` (Unix graceful)
- `kill -9 <pid>` (Unix force)
- `taskkill /PID <pid>` (Windows graceful)
- `taskkill /F /PID <pid>` (Windows force)

##### `process_name(pid: u32) -> Option<String>`

Looks up the display name of a process by PID. Returns `None` if the process doesn't exist.

---

### `utils::sanitize`

**File**: `src/utils/sanitize.rs`

#### Functions

##### `redact(input: &str) -> String`

Replaces sensitive values in text using regex patterns:

| Pattern | Example Input | Output |
|---|---|---|
| `key=value` format | `token=abc123` | `token=<redacted>` |
| `key: "value"` format | `"secret": "xyz"` | `secret=<redacted>` |

Matched key names (case-insensitive): `password`, `token`, `secret`, `apikey`.

---

### `utils::snapshot`

**File**: `src/utils/snapshot.rs`

#### Types

##### `ProcSnapshot`

```rust
pub struct ProcSnapshot {
    pub pid: u32,
    pub name: String,
    pub cmd: String,
}
```

##### `Snapshot`

```rust
pub struct Snapshot {
    pub saved_at: DateTime<Utc>,
    pub cwd: String,
    pub processes: Vec<ProcSnapshot>,
    pub env: Vec<(String, String)>,
}
```

#### Functions

##### `save_snapshot(root: &Path) -> Result<()>`

1. Enumerates all system processes via `sysinfo`.
2. Filters to processes whose command line contains the project directory path or whose name contains "cargo".
3. Captures environment variables, excluding any key containing "token" or "secret" (case-insensitive).
4. Serializes the `Snapshot` to `.devflow/snapshot.json`.

**Side effects**: Creates `.devflow/` directory if it doesn't exist.

##### `read_snapshot(root: &Path) -> Result<Snapshot>`

Reads and deserializes `.devflow/snapshot.json`.

**Errors**: File not found or JSON parse errors.
