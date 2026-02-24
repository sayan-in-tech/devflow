# devflow

> **Local-first, cross-platform developer workflow automation CLI — written in Rust.**

[![CI](https://github.com/example/devflow/actions/workflows/ci.yml/badge.svg)](https://github.com/example/devflow/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)

`devflow` is a single-binary CLI that automates the tedious parts of local development: environment validation, port diagnostics, file-watching test runners, dependency auditing, process snapshots, and an interactive TUI dashboard. It detects your project language automatically (Python, Node.js, Rust, Go) and adapts its behavior accordingly — no configuration required to get started.

---

## Table of Contents

- [devflow](#devflow)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Quick Start](#quick-start)
  - [Installation](#installation)
    - [From Source (all platforms)](#from-source-all-platforms)
    - [Pre-built Binaries](#pre-built-binaries)
    - [Homebrew (macOS)](#homebrew-macos)
    - [Windows Installer](#windows-installer)
    - [Linux Packages (.deb/.rpm)](#linux-packages-debrpm)
  - [Commands Overview](#commands-overview)
  - [Configuration](#configuration)
  - [Plugin System](#plugin-system)
  - [Documentation](#documentation)
  - [Development](#development)
    - [Prerequisites](#prerequisites)
    - [Build \& Test](#build--test)
    - [Project Structure](#project-structure)
  - [Architecture](#architecture)
  - [License](#license)

---

## Features

| Category | Capability |
|---|---|
| **Environment Health** | Validate `.env` files against a typed schema, detect missing toolchains, diagnose `PATH` issues |
| **Port Diagnostics** | Find free ports, identify which process owns a port, live-watch common dev ports |
| **File Watcher** | Recursive filesystem watcher with configurable ignore globs; auto-runs language-specific tests |
| **Dependency Audit** | Offline dependency reports for Python (`requirements.txt` / `poetry.lock`), Node (`package.json`), and Rust (`Cargo.lock`) |
| **Process Snapshots** | Save and restore a snapshot of running processes and environment variables |
| **TUI Dashboard** | Real-time terminal dashboard showing CPU, memory, and process count via `ratatui` |
| **Log Analysis** | Group and deduplicate errors from `devflow.log`, track newly seen errors across runs |
| **Plugin System** | Run external executable plugins (any language) via a JSON stdin/stdout protocol; WASM support planned |
| **Cross-Platform** | Builds and runs on Linux, macOS, and Windows; CI tests all three |

---

## Quick Start

```bash
# Clone and build
git clone https://github.com/example/devflow.git
cd devflow
cargo build --release

# Initialize a project
./target/release/devflow init      # creates .devflow.yaml
./target/release/devflow up        # check environment health
./target/release/devflow env doctor # diagnose PATH & env issues
```

Or during development:

```bash
cargo run -- init
cargo run -- up
cargo run -- env doctor
```

---

## Installation

### From Source (all platforms)

```bash
cargo install --path .
```

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/example/devflow/releases) page:

| Platform | Binary |
|---|---|
| Linux x86_64 | `devflow-linux-x86_64` |
| macOS x86_64 | `devflow-macos-x86_64` |
| Windows x86_64 | `devflow-windows-x86_64.exe` |

### Homebrew (macOS)

```bash
brew install devflow
```

### Windows Installer

Download and run `devflow-setup.exe` from the releases page, or install via winget:

```powershell
winget install Devflow.Devflow
```

### Linux Packages (.deb/.rpm)

Built via [nfpm](https://nfpm.goreleaser.com/) in CI. See [`packaging/nfpm.yaml`](packaging/nfpm.yaml).

---

## Commands Overview

| Command | Description |
|---|---|
| `devflow up` | Detect language, verify toolchain, validate env schema, report status |
| `devflow init` | Generate a default `.devflow.yaml` configuration file |
| `devflow env doctor` | Diagnose `PATH`, toolchain, and `.env` schema issues |
| `devflow env fix` | Create a minimal `.env` file if one doesn't exist |
| `devflow env diff` | Compare current `.env` against the last saved snapshot |
| `devflow port --port <N>` | Show process diagnostics for a specific port |
| `devflow port --free` | List common development ports that are currently free |
| `devflow port --watch` | Live-monitor common ports every 2 seconds |
| `devflow watch` | Watch filesystem for changes; run tests on change |
| `devflow logs` | Group and analyze errors in `devflow.log` |
| `devflow deps` | Print dependency metadata for detected project type |
| `devflow snap save` | Snapshot running processes and environment variables |
| `devflow snap restore` | Display snapshot contents for manual restoration |
| `devflow dash` | Open an interactive TUI dashboard (press `q` to quit) |
| `devflow plugin <name>` | Run a named plugin with optional `--payload JSON` |

For detailed usage information, see [docs/usage.md](docs/usage.md) and [docs/command-reference.md](docs/command-reference.md).

---

## Configuration

`devflow` uses a `.devflow.yaml` file in the project root. Run `devflow init` to generate a template:

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

For full configuration documentation, see [docs/configuration.md](docs/configuration.md).

---

## Plugin System

devflow supports external executable plugins using a simple JSON-over-stdio protocol. Plugins can be written in any language. See [docs/plugin.md](docs/plugin.md) for the full plugin development guide.

```bash
# Run the bundled infra-check plugin
devflow plugin infra-check

# Pass a JSON payload
devflow plugin my-plugin --payload '{"key": "value"}'
```

---

## Documentation

| Document | Description |
|---|---|
| [docs/usage.md](docs/usage.md) | Detailed command usage guide |
| [docs/command-reference.md](docs/command-reference.md) | Quick command reference |
| [docs/configuration.md](docs/configuration.md) | Full configuration file reference |
| [docs/plugin.md](docs/plugin.md) | Plugin development guide |
| [docs/init-walkthrough.md](docs/init-walkthrough.md) | Step-by-step initialization walkthrough |
| [docs/testing-guide.md](docs/testing-guide.md) | Testing strategy and guide |
| [docs/deployment.md](docs/deployment.md) | Build, release, and packaging guide |
| [docs/troubleshooting.md](docs/troubleshooting.md) | Common issues and solutions |
| [docs/api-reference.md](docs/api-reference.md) | Internal API and module reference |
| [docs/knowledge-transfer.md](docs/knowledge-transfer.md) | Knowledge transfer document for onboarding |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System architecture and design decisions |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution guidelines |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Community code of conduct |

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+ (2021 edition)
- Git

### Build & Test

```bash
cargo build                  # debug build
cargo build --release        # optimized release build
cargo test --all             # run all unit + integration tests
cargo fmt --all -- --check   # check formatting
cargo clippy --all-targets --all-features -- -D warnings  # lint
```

### Project Structure

```
src/
├── main.rs          # Entry point, tokio runtime, tracing setup
├── lib.rs           # Public module re-exports
├── cli.rs           # Clap CLI argument definitions
├── commands/        # One module per subcommand
│   ├── mod.rs       # Command dispatch router
│   ├── up.rs        # Environment health check
│   ├── init.rs      # Config file generation
│   ├── env.rs       # doctor / fix / diff subcommands
│   ├── port.rs      # Port diagnostics
│   ├── watch.rs     # File watcher + test runner
│   ├── logs.rs      # Log analysis
│   ├── deps.rs      # Dependency reports
│   ├── snap.rs      # Process/env snapshots
│   ├── dash.rs      # TUI dashboard
│   └── plugin.rs    # Plugin dispatch
├── plugin/
│   └── mod.rs       # Plugin resolution and execution
└── utils/
    ├── mod.rs       # Utility module re-exports
    ├── config.rs    # .devflow.yaml parsing and writing
    ├── envcheck.rs  # .env parsing and schema validation
    ├── language.rs  # Project language detection
    ├── ports.rs     # Port scanning and process lookup
    ├── sanitize.rs  # Secret redaction
    └── snapshot.rs  # Process snapshot serialization
```

---

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for a detailed breakdown of the system design, data flow, module responsibilities, and design decisions.

---

## License

This project is licensed under the [MIT License](LICENSE).
