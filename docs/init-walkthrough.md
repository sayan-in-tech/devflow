# devflow init — Step-by-Step Walkthrough

> A complete guide to initializing devflow in your project, from first run to a fully configured environment.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Step 1: Navigate to Your Project](#step-1-navigate-to-your-project)
- [Step 2: Run `devflow init`](#step-2-run-devflow-init)
- [Step 3: Edit `.devflow.yaml`](#step-3-edit-devflowyaml)
- [Step 4: Create Your `.env` File](#step-4-create-your-env-file)
- [Step 5: Run `devflow up`](#step-5-run-devflow-up)
- [Step 6: Verify with `devflow env doctor`](#step-6-verify-with-devflow-env-doctor)
- [Step 7: Add to Version Control](#step-7-add-to-version-control)
- [What's Next](#whats-next)

---

## Prerequisites

- devflow binary installed and available in `PATH` (or built locally with `cargo build`)
- A project directory with source code

---

## Step 1: Navigate to Your Project

```bash
cd /path/to/your/project
```

devflow operates on the current working directory. All commands should be run from the **project root**.

---

## Step 2: Run `devflow init`

```bash
devflow init
```

**Expected output**:
```
Created .devflow.yaml
```

This generates a `.devflow.yaml` file with template values. If the file already exists, you'll see:
```
.devflow.yaml already exists
```

---

## Step 3: Edit `.devflow.yaml`

Open the generated file and customize it for your project:

### 3a. Define Environment Variables

List every environment variable your project needs, along with its expected type:

```yaml
env:
  DATABASE_URL: string      # any non-empty value
  PORT: int                 # must be a valid integer
  DEBUG: bool               # must be "true" or "false"
  API_KEY: string           # any non-empty value
  REDIS_URL: string
```

Supported types: `string`, `int`, `bool`.

### 3b. Define Services

List the development services (for documentation and future automation):

```yaml
services:
  - name: api
    command: cargo run --bin server
  - name: worker
    command: cargo run --bin worker
  - name: database
    command: docker compose up postgres
```

### 3c. Set Start Commands

Commands to run when bootstrapping the environment:

```yaml
start_commands:
  - docker compose up -d
  - cargo build
```

### 3d. Set Test Command

Your project's test runner:

```yaml
test_command: cargo test
```

### 3e. Configure Ignore Globs

Patterns to exclude from the file watcher (`devflow watch`):

```yaml
ignore_globs:
  - target/**
  - node_modules/**
  - .git/**
  - "*.log"
```

### 3f. Declare Desired Ports

Ports your project uses:

```yaml
desired_ports:
  - 3000
  - 5432
  - 6379
```

---

## Step 4: Create Your `.env` File

Create a `.env` file with the required variables:

```bash
# Create manually:
echo "DATABASE_URL=postgres://localhost/mydb" > .env
echo "PORT=3000" >> .env
echo "DEBUG=true" >> .env
echo "API_KEY=dev-key-123" >> .env
echo "REDIS_URL=redis://localhost:6379" >> .env
```

Or use devflow to create a minimal `.env`:

```bash
devflow env fix
```

Then edit it to add actual values.

---

## Step 5: Run `devflow up`

Verify your environment:

```bash
devflow up
```

**Expected output** (healthy):
```
devflow up status
-----------------
language: Rust
toolchain: ok (/usr/bin/rustc)
expected version hint: 1.75.0
services: docker-compose file detected
env: schema matches .env
```

If there are issues:
```
env: 2 issues
 - API_KEY: missing
 - DEBUG: expected bool
recommendation: run `devflow env doctor` and `devflow env fix`
```

---

## Step 6: Verify with `devflow env doctor`

For more detailed diagnostics:

```bash
devflow env doctor
```

This checks `PATH`, toolchain availability, and all env schema violations.

---

## Step 7: Add to Version Control

Commit the config file but not the state directory:

```bash
# Add .devflow/ to .gitignore
echo ".devflow/" >> .gitignore

# Commit the config
git add .devflow.yaml .gitignore
git commit -m "feat: add devflow configuration"
```

**Do commit**: `.devflow.yaml` — shared team configuration.
**Do NOT commit**: `.devflow/` — local runtime state (snapshots, log states).
**Do NOT commit**: `.env` — contains secrets and machine-specific values.

---

## What's Next

Now that devflow is configured, try these commands:

| Command | Purpose |
|---|---|
| `devflow watch` | Auto-run tests on file changes |
| `devflow port --free` | Find available ports |
| `devflow dash` | Open the system dashboard |
| `devflow deps` | Check dependency status |
| `devflow snap save` | Save a workspace snapshot |

See the full [Usage Guide](usage.md) for detailed information on every command.
