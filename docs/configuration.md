# Configuration Reference

> Complete reference for the `.devflow.yaml` configuration file.

---

## Table of Contents

- [Overview](#overview)
- [File Location](#file-location)
- [Creating the Config](#creating-the-config)
- [Full Schema](#full-schema)
- [Field Reference](#field-reference)
  - [env](#env)
  - [services](#services)
  - [start_commands](#start_commands)
  - [test_command](#test_command)
  - [ignore_globs](#ignore_globs)
  - [desired_ports](#desired_ports)
- [Examples](#examples)
- [Validation](#validation)
- [State Directory](#state-directory)

---

## Overview

The `.devflow.yaml` file is the central configuration for devflow. It defines:

- **Environment schema**: Expected environment variables and their types.
- **Services**: Named development services with their start commands.
- **Startup commands**: Shell commands to run when bootstrapping the project.
- **Test command**: The project's test runner.
- **Ignore globs**: Patterns to exclude from the file watcher.
- **Desired ports**: Ports the project expects to use.

All fields are **optional** and have sensible defaults. Many devflow commands work without any configuration file at all.

---

## File Location

The config file must be named `.devflow.yaml` and placed in the **project root** (the directory where you run devflow commands).

```
my-project/
├── .devflow.yaml    ← here
├── .env
├── src/
└── ...
```

---

## Creating the Config

### Auto-generate with defaults

```bash
devflow init
```

This creates `.devflow.yaml` with sample values. It's idempotent — it won't overwrite an existing file.

### Manual creation

Create `.devflow.yaml` manually with any YAML editor. See the [Full Schema](#full-schema) and [Examples](#examples) below.

---

## Full Schema

```yaml
# Environment variable schema
# Map of variable name → expected type (string | int | bool)
env:
  DATABASE_URL: string
  PORT: int
  DEBUG: bool

# Named services
services:
  - name: api
    command: cargo run --release
  - name: worker
    command: python worker.py

# Commands to run on project startup
start_commands:
  - docker compose up -d
  - redis-server --daemonize yes

# Test runner command
test_command: cargo test

# Glob patterns to ignore in file watcher
ignore_globs:
  - target/**
  - node_modules/**
  - .git/**
  - "*.log"

# Ports this project expects to use
desired_ports:
  - 3000
  - 5432
  - 6379
```

---

## Field Reference

### `env`

**Type**: `map[string, string]`
**Default**: `{}`
**Used by**: `devflow up`, `devflow env doctor`

Defines expected environment variables and their types. The keys are variable names, and the values are type declarations used for validation against the `.env` file.

**Supported types**:

| Type | Validation Rule | Valid Examples | Invalid Examples |
|---|---|---|---|
| `string` | Key must exist | Any value | (missing key) |
| `int` | Must parse as `i64` | `3000`, `-1`, `0` | `abc`, `3.14`, `` |
| `bool` | Must parse as Rust `bool` | `true`, `false` | `1`, `yes`, `on` |

**Example**:

```yaml
env:
  DATABASE_URL: string
  PORT: int
  ENABLE_CACHE: bool
  API_KEY: string
```

**Validation behavior**:
- If a key is in the schema but missing from `.env`, an issue is reported: `missing`.
- If a key exists but the value doesn't match the type, an issue is reported: `expected int` or `expected bool`.
- Extra keys in `.env` that are not in the schema are ignored (no warning).

---

### `services`

**Type**: `list[{name: string, command: string}]`
**Default**: `[]`
**Used by**: Currently informational only (displayed in `devflow dash` context)

Defines named development services with their start commands.

**Example**:

```yaml
services:
  - name: api
    command: cargo run
  - name: database
    command: docker compose up postgres
  - name: frontend
    command: npm run dev
```

**Fields per service**:

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Human-readable service identifier |
| `command` | string | Yes | Shell command to start the service |

**Note**: Service definitions are currently stored in the config but not automatically orchestrated. They serve as documentation and may be used by future service management commands.

---

### `start_commands`

**Type**: `list[string]`
**Default**: `[]`
**Used by**: Currently informational only

Shell commands intended to be run when setting up the development environment. These are stored as configuration and may be used by future automation features.

**Example**:

```yaml
start_commands:
  - docker compose up -d
  - redis-server --daemonize yes
  - npm install
```

---

### `test_command`

**Type**: `string` (optional)
**Default**: `null`
**Used by**: Currently informational only

The project's test runner command. Stored as configuration for documentation purposes.

**Example**:

```yaml
test_command: cargo test
```

**Note**: The `devflow watch` command currently auto-detects the test command based on project language rather than using this field. This is a known limitation that may be addressed in future versions.

| Language | Auto-detected Test Command |
|---|---|
| Python | `pytest -q` |
| Node | `npx jest --passWithNoTests` |
| Rust | `cargo test` |
| Go | `go test ./...` |

---

### `ignore_globs`

**Type**: `list[string]`
**Default**: `[]`
**Used by**: `devflow watch`

Glob patterns for files and directories to ignore in the file watcher. Uses the `globset` crate's glob syntax.

**Example**:

```yaml
ignore_globs:
  - target/**          # Rust build output
  - node_modules/**    # Node dependencies
  - .git/**            # Git internals
  - "*.log"            # Log files
  - dist/**            # Build output
  - __pycache__/**     # Python cache
  - .venv/**           # Python virtual environments
```

**Glob syntax**:

| Pattern | Matches |
|---|---|
| `*` | Any sequence of characters (not `/`) |
| `**` | Any sequence of characters (including `/`) |
| `?` | Any single character |
| `[abc]` | Any character in the set |
| `{a,b}` | Either `a` or `b` |

**Important**: Patterns are matched relative to the project root. Use `**` for recursive matching.

---

### `desired_ports`

**Type**: `list[int]`
**Default**: `[]`
**Used by**: Currently informational only

Ports that the project expects to use during development. Stored as configuration for documentation purposes and potential future integration with `devflow port`.

**Example**:

```yaml
desired_ports:
  - 3000    # API server
  - 5173    # Vite dev server
  - 5432    # PostgreSQL
  - 6379    # Redis
```

---

## Examples

### Python Web Project

```yaml
env:
  DATABASE_URL: string
  SECRET_KEY: string
  DEBUG: bool
  PORT: int

services:
  - name: web
    command: python manage.py runserver
  - name: celery
    command: celery -A myapp worker -l info

start_commands:
  - docker compose up -d postgres redis
  - python manage.py migrate

test_command: pytest

ignore_globs:
  - __pycache__/**
  - .venv/**
  - "*.pyc"
  - .mypy_cache/**

desired_ports:
  - 8000
  - 5432
  - 6379
```

### Node.js/React Project

```yaml
env:
  REACT_APP_API_URL: string
  PORT: int
  NODE_ENV: string

services:
  - name: frontend
    command: npm start
  - name: api
    command: node server.js

start_commands:
  - npm install
  - docker compose up -d

test_command: npx jest

ignore_globs:
  - node_modules/**
  - build/**
  - coverage/**
  - dist/**

desired_ports:
  - 3000
  - 3001
```

### Rust Project

```yaml
env:
  DATABASE_URL: string
  RUST_LOG: string
  PORT: int

services:
  - name: app
    command: cargo run

start_commands:
  - docker compose up -d

test_command: cargo test

ignore_globs:
  - target/**
  - "*.log"

desired_ports:
  - 8080
  - 5432
```

### Go Microservice

```yaml
env:
  DATABASE_URL: string
  REDIS_URL: string
  PORT: int
  LOG_LEVEL: string

services:
  - name: server
    command: go run ./cmd/server
  - name: worker
    command: go run ./cmd/worker

start_commands:
  - docker compose up -d
  - go mod download

test_command: go test ./...

ignore_globs:
  - vendor/**
  - bin/**
  - "*.test"

desired_ports:
  - 8080
  - 5432
  - 6379
```

---

## Validation

devflow validates the config in two stages:

### 1. YAML Syntax (on load)

If `.devflow.yaml` contains invalid YAML, you'll see:

```
Error: invalid .devflow.yaml
```

Use a YAML linter to check syntax. Common issues:
- Tabs instead of spaces for indentation
- Missing colons after keys
- Unquoted special characters

### 2. Env Schema (on `up` / `env doctor`)

The `env` field is validated against your actual `.env` file:

```
env: 2 issues
 - PORT: missing
 - DEBUG: expected bool
```

This validation runs during `devflow up` and `devflow env doctor`.

---

## State Directory

devflow stores runtime state in the `.devflow/` directory inside your project:

```
.devflow/
├── snapshot.json           # Process/env snapshot (devflow snap save)
├── env_snapshot.json       # .env baseline (devflow env diff)
└── last_logs_state.json    # Error groups (devflow logs)
```

**Recommendation**: Add `.devflow/` to your `.gitignore`:

```gitignore
# devflow state directory
.devflow/
```

The `.devflow.yaml` config file itself **should** be committed to version control, as it defines the shared project configuration.
