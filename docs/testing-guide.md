# Testing Guide

> Complete guide to the devflow testing strategy, test organization, fixtures, and best practices.

---

## Table of Contents

- [Overview](#overview)
- [Test Organization](#test-organization)
- [Running Tests](#running-tests)
- [Unit Tests](#unit-tests)
- [Integration Tests](#integration-tests)
- [Test Fixtures](#test-fixtures)
- [Writing New Tests](#writing-new-tests)
- [Testing Patterns](#testing-patterns)
- [CI Testing](#ci-testing)
- [Debugging Tests](#debugging-tests)
- [Coverage](#coverage)

---

## Overview

devflow uses Rust's built-in test framework with these additional crates:

| Crate | Purpose |
|---|---|
| `assert_cmd` | Run the compiled binary and assert on exit code/stdout/stderr |
| `predicates` | Composable assertion matchers (e.g., `contains("text")`) |
| `tempfile` | Create temporary directories for isolated filesystem tests |

Tests are organized into three tiers:

1. **Inline unit tests** — `#[cfg(test)]` blocks within source files
2. **External unit tests** — `tests/unit/utils_tests.rs`
3. **CLI integration tests** — `tests/integration/cli_tests.rs`

---

## Test Organization

```
src/
├── utils/
│   ├── envcheck.rs       # contains #[cfg(test)] mod tests { ... }
│   ├── language.rs       # contains #[cfg(test)] mod tests { ... }
│   └── sanitize.rs       # contains #[cfg(test)] mod tests { ... }
│
tests/
├── unit.rs               # test harness entry: #[path = "unit/utils_tests.rs"]
├── unit/
│   └── utils_tests.rs    # external unit tests for utils modules
├── integration.rs         # test harness entry: #[path = "integration/cli_tests.rs"]
├── integration/
│   └── cli_tests.rs      # CLI binary integration tests
└── fixtures/
    ├── node_repo/         # package.json
    ├── python_repo/       # pyproject.toml
    └── rust_repo/         # Cargo.toml
```

### Why Two Levels?

- **Inline tests** (`#[cfg(test)]`) test private functions and internal logic that isn't exposed through the public API.
- **External unit tests** (`tests/unit/`) test public library APIs by importing `devflow::utils::*`.
- **Integration tests** (`tests/integration/`) test the full binary as a user would invoke it, verifying argument parsing, output format, and exit codes.

---

## Running Tests

### All Tests

```bash
cargo test --all
```

### By Category

```bash
# Inline unit tests (within src/)
cargo test --lib

# External unit tests (tests/unit/)
cargo test --test unit

# Integration tests (tests/integration/)
cargo test --test integration
```

### A Specific Test

```bash
cargo test test_name

# Example
cargo test redact_masks_secret_values
cargo test init_creates_config
```

### With Output (see println! during tests)

```bash
cargo test -- --nocapture
```

### Verbose (see each test name)

```bash
cargo test -- --show-output
```

---

## Unit Tests

### Inline Unit Tests

Located inside source files, within `#[cfg(test)] mod tests` blocks.

#### `src/utils/envcheck.rs`

```rust
#[test]
fn validates_int_type() {
    let mut schema = HashMap::new();
    schema.insert("PORT".to_string(), "int".to_string());
    let mut actual = HashMap::new();
    actual.insert("PORT".to_string(), "abc".to_string());
    let issues = validate_env_schema(&schema, &actual);
    assert_eq!(issues.len(), 1);
}
```

Tests that the env schema validator correctly flags non-integer values for `int`-typed keys.

#### `src/utils/language.rs`

```rust
#[test]
fn detects_rust() {
    let dir = tempdir().expect("tempdir");
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname='a'\n").expect("write");
    assert_eq!(detect_project_language(dir.path()), Language::Rust);
}
```

Creates a temp directory with a `Cargo.toml` and verifies Rust is detected.

#### `src/utils/sanitize.rs`

```rust
#[test]
fn redacts_basic_secret() {
    let out = redact("token=abc123");
    assert!(out.contains("token=<redacted>"));
    assert!(!out.contains("abc123"));
}
```

Verifies that secret values are replaced with `<redacted>`.

### External Unit Tests (`tests/unit/utils_tests.rs`)

These test the public API surface:

```rust
#[test]
fn redact_masks_secret_values() {
    let out = redact("password=supersecret");
    assert!(out.contains("password=<redacted>"));
}

#[test]
fn env_schema_detects_missing_key() {
    let schema = HashMap::from([("PORT".to_string(), "int".to_string())]);
    let actual = HashMap::new();
    let issues = validate_env_schema(&schema, &actual);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].key, "PORT");
}

#[test]
fn language_detects_node_repo() {
    let td = tempdir().expect("tempdir");
    std::fs::write(td.path().join("package.json"), "{}").expect("write");
    assert_eq!(detect_project_language(td.path()), Language::Node);
}
```

---

## Integration Tests

### CLI Integration Tests (`tests/integration/cli_tests.rs`)

These invoke the compiled `devflow` binary as a subprocess.

#### `init_creates_config`

```rust
#[test]
fn init_creates_config() {
    let td = tempfile::tempdir().expect("tempdir");
    cargo_bin_cmd!("devflow")
        .current_dir(td.path())
        .arg("init")
        .assert()
        .success()
        .stdout(contains("Created .devflow.yaml"));
    assert!(td.path().join(".devflow.yaml").exists());
}
```

Verifies that:
1. `devflow init` exits with code 0.
2. stdout contains the expected confirmation message.
3. The `.devflow.yaml` file was actually created.

#### `port_free_outputs_json_list`

```rust
#[test]
fn port_free_outputs_json_list() {
    let td = tempfile::tempdir().expect("tempdir");
    cargo_bin_cmd!("devflow")
        .current_dir(td.path())
        .args(["port", "--free"])
        .assert()
        .success()
        .stdout(contains("["));
}
```

Verifies that `devflow port --free` outputs a JSON array.

---

## Test Fixtures

Located in `tests/fixtures/`, these provide minimal project directories for testing language detection:

| Directory | Contents | Detects As |
|---|---|---|
| `tests/fixtures/node_repo/` | `package.json` | Node |
| `tests/fixtures/python_repo/` | `pyproject.toml` | Python |
| `tests/fixtures/rust_repo/` | `Cargo.toml` | Rust |

These are used for testing `detect_project_language()` against real directory structures rather than programmatically created temp directories.

---

## Writing New Tests

### Adding a Unit Test for a Utility Function

1. Open the source file (e.g., `src/utils/ports.rs`).
2. Add or extend the `#[cfg(test)] mod tests` block at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_free_ports_returns_valid_list() {
        let ports = common_free_ports();
        // All returned ports should be in the known list
        for p in &ports {
            assert!([3000, 3001, 5173, 8000, 8080, 5432, 6379].contains(p));
        }
    }
}
```

### Adding an External Unit Test

Add a test to `tests/unit/utils_tests.rs`:

```rust
#[test]
fn config_default_has_expected_fields() {
    use devflow::utils::config::DevflowConfig;
    let cfg = DevflowConfig::default();
    assert!(cfg.env.is_empty());
    assert!(cfg.services.is_empty());
}
```

### Adding a CLI Integration Test

Add a test to `tests/integration/cli_tests.rs`:

```rust
#[test]
fn deps_runs_without_error() {
    let td = tempfile::tempdir().expect("tempdir");
    // Create a Cargo.toml so it detects as Rust
    std::fs::write(td.path().join("Cargo.toml"), "[package]\nname=\"test\"\n")
        .expect("write");
    cargo_bin_cmd!("devflow")
        .current_dir(td.path())
        .arg("deps")
        .assert()
        .success()
        .stdout(contains("rust deps"));
}
```

### Adding a Test Fixture

1. Create a new directory under `tests/fixtures/` (e.g., `tests/fixtures/go_repo/`).
2. Add the minimum marker file (e.g., `go.mod`).
3. Reference it in your tests.

---

## Testing Patterns

### Pattern: Temp Directory Isolation

Always use `tempfile::tempdir()` for tests that create files:

```rust
let td = tempdir().expect("tempdir");
std::fs::write(td.path().join("Cargo.toml"), "...").expect("write");
// td is cleaned up when dropped
```

### Pattern: Assert on Exit Code + Output

```rust
cargo_bin_cmd!("devflow")
    .arg("init")
    .current_dir(td.path())
    .assert()
    .success()                          // exit code 0
    .stdout(contains("Created"));       // stdout check
```

### Pattern: Assert on File Side Effects

```rust
cargo_bin_cmd!("devflow")
    .arg("init")
    .current_dir(td.path())
    .assert()
    .success();

// Verify the side effect
assert!(td.path().join(".devflow.yaml").exists());
let content = std::fs::read_to_string(td.path().join(".devflow.yaml")).unwrap();
assert!(content.contains("env:"));
```

### Pattern: Testing Error Cases

```rust
cargo_bin_cmd!("devflow")
    .arg("nonexistent-command")
    .assert()
    .failure();                         // exit code != 0
```

---

## CI Testing

Tests run on every push and PR via `.github/workflows/ci.yml`:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
steps:
  - cargo fmt --all -- --check
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo test --all
```

All three platforms must pass. The Rust cache (`Swatinem/rust-cache@v2`) speeds up builds.

---

## Debugging Tests

### See Test Output

```bash
cargo test -- --nocapture
```

### Run a Single Failing Test with Backtrace

```bash
RUST_BACKTRACE=1 cargo test test_name -- --nocapture
```

### Build the Test Binary Without Running

```bash
cargo test --no-run
```

### Run Integration Tests Against a Specific Binary

The `cargo_bin_cmd!` macro automatically finds the built binary. If you need to test against a release build:

```bash
cargo test --release --test integration
```

---

## Coverage

### Current Coverage Summary

| Area | Status | Priority |
|---|---|---|
| Env schema validation | Covered | — |
| Language detection (Rust, Node) | Covered | — |
| Secret redaction | Covered | — |
| CLI init command | Covered (integration) | — |
| CLI port --free | Covered (integration) | — |
| Language detection (Python, Go) | Not covered | Medium |
| Config loading/writing | Not covered | Medium |
| Snapshot save/restore | Not covered | Medium |
| Log analysis | Not covered | Low |
| Plugin dispatch | Not covered | High |
| Dashboard (TUI) | Not covered | Low (hard to test) |

### Improving Coverage

Priority areas for new tests:
1. **Plugin dispatch**: Mock a simple plugin script and verify the JSON round-trip.
2. **Config YAML round-trip**: Write a config, read it back, verify equivalence.
3. **Snapshot serialization**: Save a snapshot, read it back, verify structure.
4. **All language detection variants**: Add tests for Python (`pyproject.toml`) and Go (`go.mod`).
5. **`env diff`**: Create two `.env` files, run diff, verify add/change/remove detection.
