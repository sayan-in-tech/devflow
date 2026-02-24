# Knowledge Transfer Document — devflow

> Comprehensive onboarding document for engineers joining the devflow project. This document covers everything needed to understand, develop, debug, and extend the codebase.

**Last updated**: February 2026
**Target audience**: New contributors, onboarding engineers, team members taking over maintenance

---

## Table of Contents

1. [Project Summary](#1-project-summary)
2. [Business Context](#2-business-context)
3. [Technical Stack](#3-technical-stack)
4. [Repository Layout](#4-repository-layout)
5. [Build & Run](#5-build--run)
6. [Architecture Deep Dive](#6-architecture-deep-dive)
7. [Configuration System](#7-configuration-system)
8. [Command-by-Command Walkthrough](#8-command-by-command-walkthrough)
9. [Plugin System](#9-plugin-system)
10. [Testing Strategy](#10-testing-strategy)
11. [CI/CD Pipeline](#11-cicd-pipeline)
12. [Packaging & Distribution](#12-packaging--distribution)
13. [State Files & Side Effects](#13-state-files--side-effects)
14. [Security Considerations](#14-security-considerations)
15. [Known Limitations](#15-known-limitations)
16. [Future Roadmap](#16-future-roadmap)
17. [Common Tasks](#17-common-tasks)
18. [Glossary](#18-glossary)

---

## 1. Project Summary

**devflow** is a local-first, cross-platform CLI tool that automates day-to-day developer workflow tasks. Written in Rust, it compiles to a single binary with zero runtime dependencies.

**Core value proposition**: A single command (`devflow up`) tells a developer if their local environment is healthy — correct toolchain installed, `.env` file valid, services running, ports available. Complementary commands provide port diagnostics, file-watching test runners, dependency auditing, process snapshots, and an interactive dashboard.

**Key characteristics**:
- No network calls (100% local)
- No daemon or background service
- Cross-platform (Linux, macOS, Windows)
- Auto-detects project language (Python, Node.js, Rust, Go)
- Extensible via executable plugins (any language)

---

## 2. Business Context

### Problem Statement

Developers frequently encounter "works on my machine" issues caused by:
- Missing or wrong toolchain versions
- Missing environment variables
- Port conflicts with stale processes
- Unknown dependency states
- No quick way to snapshot and restore a working state

### Solution

devflow provides a standardized, team-shareable way to:
1. **Define** the expected development environment in `.devflow.yaml`
2. **Validate** the local machine against that definition
3. **Diagnose** and **fix** common issues automatically
4. **Monitor** the local workspace in real-time

### Target Users

- Individual developers setting up new projects
- Teams wanting consistent local environments
- DevOps engineers creating onboarding tooling
- Open-source maintainers providing contributor setup scripts

---

## 3. Technical Stack

| Component | Technology | Version | Notes |
|---|---|---|---|
| Language | Rust | 2021 edition | Minimum rustc ~1.70 |
| Async Runtime | tokio | 1.x | Full features enabled |
| CLI Framework | clap | 4.5 | Derive API |
| TUI Framework | ratatui + crossterm | 0.28 | Terminal dashboard |
| System Info | sysinfo | 0.33 | Cross-platform process/CPU/memory |
| File Watching | notify | 6.x | Uses OS-native APIs |
| Serialization | serde + serde_yaml + serde_json | 1.x / 0.9 / 1.x | Config and data |
| Error Handling | anyhow | 1.x | Ergonomic error chains |
| Logging | tracing + tracing-subscriber | 0.1 / 0.3 | Structured, `RUST_LOG` controlled |
| Glob Matching | globset | 0.4 | Ignore patterns |
| Regex | regex | 1.x | Secret redaction |
| Path Lookup | which | 7.x | Find executables in PATH |
| Testing | assert_cmd + predicates + tempfile | 2/3/3 | CLI integration tests |

---

## 4. Repository Layout

```
devflow/
├── Cargo.toml                    # Crate metadata, dependencies
├── README.md                     # Project overview and quick start
├── ARCHITECTURE.md               # System design document
├── CONTRIBUTING.md               # Contribution guidelines
├── CODE_OF_CONDUCT.md            # Community standards
│
├── src/
│   ├── main.rs                   # Entry point: tracing init, CLI parse, dispatch
│   ├── lib.rs                    # Public module re-exports
│   ├── cli.rs                    # Clap CLI definitions (all args/commands)
│   │
│   ├── commands/                 # One file per CLI subcommand
│   │   ├── mod.rs                # Command router (match → dispatch)
│   │   ├── up.rs                 # `devflow up` — environment health check
│   │   ├── init.rs               # `devflow init` — generate .devflow.yaml
│   │   ├── env.rs                # `devflow env doctor|fix|diff`
│   │   ├── port.rs               # `devflow port` — port diagnostics
│   │   ├── watch.rs              # `devflow watch` — file watcher + test runner
│   │   ├── logs.rs               # `devflow logs` — log error analysis
│   │   ├── deps.rs               # `devflow deps` — dependency reports
│   │   ├── snap.rs               # `devflow snap save|restore`
│   │   ├── dash.rs               # `devflow dash` — TUI dashboard
│   │   └── plugin.rs             # `devflow plugin <name>` — plugin entry
│   │
│   ├── plugin/
│   │   └── mod.rs                # Plugin resolution, subprocess execution, JSON protocol
│   │
│   └── utils/
│       ├── mod.rs                # Re-exports all util modules
│       ├── config.rs             # DevflowConfig: load/write .devflow.yaml
│       ├── envcheck.rs           # .env parser, schema validator, PATH diagnostics
│       ├── language.rs           # Project language detection (marker files)
│       ├── ports.rs              # Port scanning, process lookup, kill suggestions
│       ├── sanitize.rs           # Regex-based secret redaction
│       └── snapshot.rs           # Process/env snapshot capture and restoration
│
├── tests/
│   ├── integration.rs            # Integration test entry (includes cli_tests)
│   ├── unit.rs                   # Unit test entry (includes utils_tests)
│   ├── integration/
│   │   └── cli_tests.rs          # CLI binary integration tests
│   ├── unit/
│   │   └── utils_tests.rs        # Utility function unit tests
│   └── fixtures/
│       ├── node_repo/            # package.json (language detection fixture)
│       ├── python_repo/          # pyproject.toml (language detection fixture)
│       └── rust_repo/            # Cargo.toml (language detection fixture)
│
├── docs/
│   ├── usage.md                  # Detailed command usage guide
│   ├── command-reference.md      # Quick command reference
│   ├── configuration.md          # Full config file reference
│   ├── plugin.md                 # Plugin development guide
│   ├── init-walkthrough.md       # Init walkthrough
│   ├── api-reference.md          # Internal API documentation
│   ├── testing-guide.md          # Testing strategy guide
│   ├── deployment.md             # Build, release, packaging
│   ├── troubleshooting.md        # Common issues & solutions
│   └── knowledge-transfer.md     # This document
│
├── plugins/
│   └── devflow-plugin-infra-check.py   # Example Python plugin
│
├── examples/
│   └── sample.devflow.yaml       # Example configuration file
│
├── packaging/
│   ├── nfpm.yaml                 # Linux .deb/.rpm packaging config
│   ├── windows/
│   │   └── installer.iss         # InnoSetup Windows installer script
│   └── winget/
│       └── devflow.yaml          # Winget package manifest
│
├── scripts/
│   ├── build-packages.sh         # Release build + packaging script
│   └── generate-homebrew-formula.sh  # Homebrew formula generator
│
└── .github/workflows/
    ├── ci.yml                    # CI: fmt + clippy + test on Linux/macOS/Windows
    └── release.yml               # Release: build binaries + upload artifacts
```

---

## 5. Build & Run

### First-time Setup

```bash
# Install Rust via rustup (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/example/devflow.git
cd devflow

# Build (debug mode)
cargo build

# Run
cargo run -- --help
```

### Common Development Commands

```bash
# Run a specific command
cargo run -- up
cargo run -- init
cargo run -- env doctor
cargo run -- port --free
cargo run -- watch
cargo run -- logs
cargo run -- deps
cargo run -- snap save
cargo run -- dash
cargo run -- plugin infra-check

# Run tests
cargo test --all

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Release build
cargo build --release
```

### Environment Variables

| Variable | Purpose | Default |
|---|---|---|
| `RUST_LOG` | Controls tracing log level | (none — logs disabled) |

Examples:
```bash
RUST_LOG=debug cargo run -- up     # verbose logging
RUST_LOG=devflow=trace cargo run -- watch  # trace only devflow crate
```

---

## 6. Architecture Deep Dive

### Execution Flow

```
User types: devflow up
    │
    ▼
main.rs
    ├── Initialize tracing-subscriber (reads RUST_LOG env var)
    ├── Cli::parse() — clap parses argv
    └── commands::run(cli) — async dispatch
           │
           ▼
    commands::up::run()
           │
           ├── env::current_dir() — get project root
           ├── language::detect_project_language(&root)
           ├── which::which("rustc") — check toolchain
           ├── language::expected_toolchain_hint(&root)
           ├── check docker-compose.yml/compose.yaml
           └── if .devflow.yaml exists:
                  ├── config::load_config(&root)
                  ├── envcheck::parse_dotenv(&root)
                  └── envcheck::validate_env_schema(...)
```

### Module Dependency Graph

```
main.rs
  └── lib::commands::run()
         ├── commands::up
         │      └── utils::{config, envcheck, language}
         ├── commands::init
         │      └── utils::config
         ├── commands::env
         │      └── utils::{config, envcheck}
         ├── commands::port
         │      └── utils::ports
         ├── commands::watch
         │      └── utils::{config, language}
         ├── commands::logs
         │      └── (standalone — uses chrono, serde_json)
         ├── commands::deps
         │      └── utils::language
         ├── commands::snap
         │      └── utils::snapshot
         ├── commands::dash
         │      └── (standalone — uses ratatui, sysinfo)
         └── commands::plugin
                └── plugin::dispatch
```

### Data Structures

The central configuration type is `DevflowConfig` (in `utils/config.rs`):

```yaml
# .devflow.yaml maps to:
env:              # HashMap<String, String> — key → type
services:         # Vec<ServiceDef> — name + command
start_commands:   # Vec<String> — shell commands
test_command:     # Option<String> — test runner
ignore_globs:     # Vec<String> — watch ignore patterns
desired_ports:    # Vec<u16> — expected ports
```

### Error Handling Pattern

All functions return `anyhow::Result<T>`. Errors are enriched with `.context()`:

```rust
fs::read_to_string(&path)
    .with_context(|| format!("could not read {}", path.display()))?;
```

Errors propagate to `main()`, which prints the full error chain and exits with code 1.

---

## 7. Configuration System

### File: `.devflow.yaml`

Located in the project root. Created by `devflow init`. Optional for most commands (they degrade gracefully when absent).

### Schema

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `env` | `map[string → string]` | No | `{}` | Env var name → expected type (`string`, `int`, `bool`) |
| `services` | `list[{name, command}]` | No | `[]` | Named service definitions |
| `start_commands` | `list[string]` | No | `[]` | Commands to run on project startup |
| `test_command` | `string` | No | `null` | Test runner command |
| `ignore_globs` | `list[string]` | No | `[]` | Glob patterns to ignore in file watcher |
| `desired_ports` | `list[int]` | No | `[]` | Ports this project expects to use |

### Default Generated Config

```yaml
env:
  DATABASE_URL: string
  PORT: int
services:
- name: app
  command: cargo run
start_commands:
- docker compose up -d
test_command: cargo test
ignore_globs:
- target/**
- node_modules/**
desired_ports:
- 3000
- 5432
```

---

## 8. Command-by-Command Walkthrough

### `devflow up`

**Purpose**: Quick health check of the local development environment.

**What it does**:
1. Prints the detected project language (Python/Node/Go/Rust/Unknown).
2. Checks if the matching toolchain binary exists in `PATH`.
3. Shows the expected version hint (from `.nvmrc`, `rust-toolchain`, etc.).
4. Checks for Docker Compose files.
5. If `.devflow.yaml` exists, validates `.env` against the schema and reports issues.

**When to use**: After cloning a repo, after switching branches, as a morning sanity check.

**Example output**:
```
devflow up status
-----------------
language: Rust
toolchain: ok (C:\Users\dev\.cargo\bin\rustc.exe)
expected version hint: 1.75.0
services: docker-compose file detected
env: 1 issues
 - PORT: missing
recommendation: run `devflow env doctor` and `devflow env fix`
```

---

### `devflow init`

**Purpose**: Generate a starter `.devflow.yaml`.

**Idempotent**: Won't overwrite an existing config.

**When to use**: Once, when setting up devflow in a new project.

---

### `devflow env doctor`

**Purpose**: Comprehensive environment diagnostics.

**What it checks**:
1. Is `PATH` set?
2. Is Python available?
3. Is Node available?
4. Does `.env` match the schema in `.devflow.yaml`?

**When to use**: When `devflow up` reports env issues.

---

### `devflow env fix`

**Purpose**: Create a minimal `.env` file.

**What it does**: If `.env` doesn't exist, creates one with a comment header. Does nothing if the file exists.

---

### `devflow env diff`

**Purpose**: Track changes to your `.env` file over time.

**How it works**:
- First run: saves current `.env` as a baseline to `.devflow/env_snapshot.json`.
- Subsequent runs: compares current `.env` to the baseline and prints added/changed/removed keys.

---

### `devflow port --free`

**Purpose**: Find available ports.

**Output**: JSON array of free common development ports.

```json
[3000, 3001, 5173, 8000, 8080]
```

---

### `devflow port --port 3000`

**Purpose**: Diagnose what's using a specific port.

**Output**:
```
Port 3000 is owned by pid 12345
parent pid: 12340
cmd: node server.js
mem: 45678 KB
uptime: 120 sec
tip: Try graceful stop first: kill 12345
tip: Windows graceful: taskkill /PID 12345
```

---

### `devflow port --watch`

**Purpose**: Continuously monitor common development ports.

**Behavior**: Polls ports 3000, 5173, 5432, 6379, 8080 every 2 seconds. Runs until `Ctrl+C`.

---

### `devflow watch`

**Purpose**: Automatically run tests when files change.

**How it works**:
1. Sets up a filesystem watcher on the entire project directory.
2. Ignores paths matching `ignore_globs` from `.devflow.yaml`.
3. When files change, runs the appropriate test command for the detected language.
4. Runs until `Ctrl+C`.

---

### `devflow logs`

**Purpose**: Analyze error logs.

**How it works**:
1. Reads `devflow.log` from the project root.
2. Filters lines with `ERROR` or `panic`.
3. Groups similar errors (normalizes numbers → `<n>`).
4. Shows frequency of each error group.
5. Highlights errors not seen in the previous run.

---

### `devflow deps`

**Purpose**: Quick dependency audit (offline).

**Language-specific reports**:
- Python: checks for `requirements.txt`, `poetry.lock`
- Node: counts dependencies in `package.json`
- Rust: checks for `Cargo.lock`, suggests `cargo deny`

---

### `devflow snap save`

**Purpose**: Capture current workspace state.

**What it captures**:
- All processes related to the project directory
- Environment variables (secrets excluded)
- Timestamp

**Output**: `.devflow/snapshot.json`

---

### `devflow snap restore`

**Purpose**: View a saved snapshot.

**Note**: Does NOT automatically restart processes. It shows what was captured so you can manually restore.

---

### `devflow dash`

**Purpose**: Real-time system dashboard in the terminal.

**Features**:
- CPU usage percentage
- Memory used (KB)
- Process count
- Press `q` to exit

**Tech**: Built with `ratatui` + `crossterm`.

---

### `devflow plugin <name>`

**Purpose**: Run an external plugin.

**Options**: `--payload '{"key": "value"}'` — pass JSON data to the plugin.

**Example**: `devflow plugin infra-check`

---

## 9. Plugin System

### Overview

Plugins are external executables that communicate with devflow via JSON over stdin/stdout. They can be written in any language.

### Protocol

**Input** (stdin):
```json
{ "command": "plugin-name", "payload": {} }
```

**Output** (stdout):
```json
{ "ok": true, "message": "description", "data": {} }
```

### Example Plugin (Python)

`plugins/devflow-plugin-infra-check.py`:
- Reads stdin JSON
- Checks for `AWS_PROFILE` and `KUBECONFIG` env vars
- Returns success/failure with missing credential list

### Plugin Resolution

1. `which devflow-plugin-<name>` (searches PATH)
2. `./plugins/devflow-plugin-<name>` (project-local)

### Writing a New Plugin

1. Create an executable named `devflow-plugin-<your-name>`
2. Read JSON from stdin
3. Write JSON response to stdout
4. Exit 0 on success, non-zero on failure
5. Place in `./plugins/` or install globally

---

## 10. Testing Strategy

### Test Organization

| Location | Type | Framework |
|---|---|---|
| `src/**` (`#[cfg(test)]` blocks) | Inline unit tests | std test runner |
| `tests/unit/utils_tests.rs` | External unit tests | std + tempfile |
| `tests/integration/cli_tests.rs` | CLI integration tests | assert_cmd + predicates |
| `tests/fixtures/` | Test data | Marker files for language detection |

### Running Tests

```bash
cargo test --all          # everything
cargo test --lib          # inline unit tests only
cargo test --test unit    # external unit tests
cargo test --test integration  # CLI integration tests
```

### Current Test Coverage

| Module | Coverage | Notes |
|---|---|---|
| `envcheck` | Good | Validates int type checking, missing keys |
| `language` | Good | Tests Rust and Node detection |
| `sanitize` | Good | Tests password/token redaction |
| `cli (init)` | Good | Integration test verifies file creation |
| `cli (port)` | Good | Integration test verifies JSON output |

### Adding Tests

See the [Testing Guide](testing-guide.md) for detailed instructions on writing and organizing tests.

---

## 11. CI/CD Pipeline

### CI (`.github/workflows/ci.yml`)

**Triggers**: Every push, every pull request.

**Matrix**: `ubuntu-latest`, `macos-latest`, `windows-latest`.

**Steps**:
1. Checkout code
2. Install stable Rust toolchain
3. Cache Rust dependencies (Swatinem/rust-cache)
4. `cargo fmt --all -- --check`
5. `cargo clippy --all-targets --all-features -- -D warnings`
6. `cargo test --all`

### Release (`.github/workflows/release.yml`)

**Triggers**: Push of `v*` tag, manual dispatch.

**Matrix**:
| OS | Target | Artifact |
|---|---|---|
| ubuntu-latest | `x86_64-unknown-linux-gnu` | `devflow-linux-x86_64` |
| macos-latest | `x86_64-apple-darwin` | `devflow-macos-x86_64` |
| windows-latest | `x86_64-pc-windows-msvc` | `devflow-windows-x86_64.exe` |

**Steps**:
1. Build release binary for target
2. Upload artifact
3. (Publish job) Generate Homebrew formula

---

## 12. Packaging & Distribution

| Format | Config File | Tool |
|---|---|---|
| Linux .deb/.rpm | `packaging/nfpm.yaml` | nfpm |
| Windows Installer | `packaging/windows/installer.iss` | InnoSetup |
| Windows winget | `packaging/winget/devflow.yaml` | winget CLI |
| macOS Homebrew | Generated by `scripts/generate-homebrew-formula.sh` | Homebrew |
| Raw binary | `scripts/build-packages.sh` | cargo |

---

## 13. State Files & Side Effects

devflow stores per-project state in `.devflow/` within the project root:

| File | Written By | Purpose |
|---|---|---|
| `.devflow/snapshot.json` | `devflow snap save` | Process/env snapshot |
| `.devflow/env_snapshot.json` | `devflow env diff` | `.env` baseline for diffing |
| `.devflow/last_logs_state.json` | `devflow logs` | Previous error groups for change detection |

The root file `.devflow.yaml` is written by `devflow init`.

The `.env` file may be created by `devflow env fix`.

**Recommendation**: Add `.devflow/` to `.gitignore`.

---

## 14. Security Considerations

| Area | Implementation |
|---|---|
| Secret redaction | `sanitize::redact()` strips password/token/secret/apikey from text output |
| Snapshot security | `save_snapshot()` excludes env vars with "token" or "secret" in the key name |
| Plugin trust | Plugins are external executables — devflow does not sandbox them. Only use trusted plugins |
| No network | devflow makes zero HTTP requests. No telemetry, no package registry calls |
| File scope | All reads/writes are within the project directory tree |

---

## 15. Known Limitations

1. **Port detection is heuristic**: `find_owner_by_port()` matches port numbers as substrings in process names/arguments. This can produce false positives (e.g., PID 3000 matching port 3000).

2. **Snap restore is advisory**: `devflow snap restore` only prints what would be restored. It does not restart processes.

3. **No multi-language projects**: Language detection returns the first match. A project with both `Cargo.toml` and `package.json` is detected as Node (since Node has higher priority than Rust).

4. **Go dependency analysis not implemented**: `devflow deps` for Go projects prints "not yet available".

5. **WASM plugins not yet functional**: Plugin names ending in `.wasm` are recognized but bail immediately.

6. **No `.env` file format support for quoted values or multi-line values**: The `parse_dotenv()` function does a simple `split('=')`.

7. **Logs command requires a specific log file**: Only reads `devflow.log` from the project root. Doesn't integrate with system logs or other log formats.

---

## 16. Future Roadmap

Based on code comments, architecture decisions, and TODOs:

- **WASM plugin runtime**: Add `wasmtime` support for sandboxed plugins
- **Go dependency analysis**: Implement `go list -m all` parsing
- **Network-optional features**: Package version checking, vulnerability scanning (opt-in)
- **Service orchestration**: Actually start/stop services defined in `.devflow.yaml`
- **Config inheritance**: Support workspace-level and project-level configs
- **Shell completions**: Generate bash/zsh/fish completions via clap
- **`devflow doctor` subcommand for deps**: License risk scanning integration

---

## 17. Common Tasks

### "I want to add a new CLI flag to an existing command"

1. Add the field to the corresponding `Args` struct in `src/cli.rs`.
2. Use the field in the command's `run()` function.
3. Add a test in `tests/integration/cli_tests.rs`.
4. Update `docs/command-reference.md` and `docs/usage.md`.

### "I want to support a new project language"

1. Add a variant to the `Language` enum in `src/utils/language.rs`.
2. Add the marker file check in `detect_project_language()`.
3. Add a case in `commands/watch.rs::run_impacted_tests()`.
4. Add a report function in `commands/deps.rs`.
5. Add a test fixture in `tests/fixtures/`.

### "I want to add a new config field"

1. Add the field to `DevflowConfig` in `src/utils/config.rs` (with `#[serde(default)]`).
2. Update `write_default_config()` if it should appear in generated configs.
3. Use the field in the relevant command.
4. Update `docs/configuration.md` and `examples/sample.devflow.yaml`.

### "I want to debug a specific command"

```bash
RUST_LOG=debug cargo run -- <command>
```

### "I want to create a release"

```bash
git tag v0.2.0
git push --tags
# CI handles the rest
```

---

## 18. Glossary

| Term | Definition |
|---|---|
| **devflow** | The CLI tool itself |
| **`.devflow.yaml`** | Per-project configuration file defining env schema, services, and settings |
| **`.devflow/`** | Hidden directory for devflow state files (snapshots, log state, env baselines) |
| **Marker file** | A file whose presence indicates a project language (e.g., `Cargo.toml` → Rust) |
| **Env schema** | The `env` section of `.devflow.yaml` mapping env var names to expected types |
| **Plugin** | An external executable that communicates via JSON stdin/stdout |
| **Snapshot** | A captured state of processes and env vars at a point in time |
| **TUI** | Terminal User Interface — the `devflow dash` interactive dashboard |
| **nfpm** | A tool for creating Linux packages (.deb, .rpm) without a full build system |
| **InnoSetup** | A Windows installer creator (.exe setup packages) |
