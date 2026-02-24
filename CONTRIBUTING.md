# Contributing to devflow

Thank you for considering contributing to devflow! This guide will help you get started.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Testing Requirements](#testing-requirements)
- [Commit Message Convention](#commit-message-convention)
- [Pull Request Process](#pull-request-process)
- [Adding a New Command](#adding-a-new-command)
- [Adding a New Utility Module](#adding-a-new-utility-module)
- [Writing a Plugin](#writing-a-plugin)
- [Release Process](#release-process)

---

## Code of Conduct

This project adheres to the [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold respectful and inclusive behavior.

---

## Getting Started

1. **Fork** the repository on GitHub.
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/<your-username>/devflow.git
   cd devflow
   ```
3. **Create a branch** for your change:
   ```bash
   git checkout -b feat/my-feature
   ```

---

## Development Setup

### Prerequisites

| Tool | Minimum Version | Purpose |
|---|---|---|
| Rust (rustup) | 1.70+ | Compilation (2021 edition) |
| Git | 2.x | Version control |
| Python 3 | 3.8+ | Running plugin examples / testing plugin protocol |
| Node.js | 18+ | Optional, for testing Node language detection |

### Build

```bash
cargo build              # debug build
cargo build --release    # optimized build
```

### Verify

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

All three checks must pass before submitting a PR.

---

## Development Workflow

1. **Check existing issues** before starting. If your change is non-trivial, open an issue first to discuss the approach.
2. **Write code** following the [Code Standards](#code-standards) below.
3. **Write or update tests** for every behavior change. See [Testing Requirements](#testing-requirements).
4. **Run the full check suite** (`fmt`, `clippy`, `test`).
5. **Commit** using the [Commit Message Convention](#commit-message-convention).
6. **Push** and open a Pull Request.

---

## Code Standards

### Rust Style

- **Format**: All code must pass `cargo fmt`. The project uses default rustfmt settings.
- **Lint**: All code must pass `cargo clippy` with `-D warnings` (warnings are errors).
- **Edition**: 2021.
- **Error handling**: Use `anyhow::Result` for all fallible functions. Attach context with `.context()` or `.with_context()`. Never use `.unwrap()` in non-test code.
- **Async**: All command handlers are `async fn` for uniformity, even if the body is synchronous.
- **Naming**: Follow Rust naming conventions — `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE` for constants.

### Design Principles

1. **Local-first**: No network calls. No telemetry. No external services.
2. **Deterministic**: Same inputs must produce the same outputs. Avoid time-dependent behavior except for timestamps in snapshots/logs.
3. **Secret-safe**: Never log, print, or snapshot raw secrets. Use `sanitize::redact()` where needed.
4. **Cross-platform**: All code must work on Linux, macOS, and Windows. Use `std::path::Path`, not hardcoded separators. Test on all three in CI.
5. **Minimal config**: Zero-config should work for basic use cases. `.devflow.yaml` is optional for most commands.

### File Organization

- One command per file in `src/commands/`.
- Shared logic goes in `src/utils/` as a dedicated module.
- Plugin runtime logic lives in `src/plugin/`.
- Each new util module must be re-exported in `src/utils/mod.rs`.

---

## Testing Requirements

### Unit Tests

- Place unit tests in the same file as the code, inside a `#[cfg(test)] mod tests` block.
- Test public functions and important internal logic.
- Use `tempfile::tempdir()` for any tests that touch the filesystem.

### Integration Tests

- Place integration tests in `tests/integration/`.
- Use `assert_cmd` and `predicates` for CLI testing.
- Integration tests should invoke the compiled binary, not library functions directly.

### Test Fixtures

- Place fixture files in `tests/fixtures/`.
- Existing fixtures: `node_repo/`, `python_repo/`, `rust_repo/` (each containing the minimum marker file for language detection).

### Running Tests

```bash
# All tests
cargo test --all

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# A specific test
cargo test test_name
```

### Coverage Expectations

All PRs should:
- Add tests for any new public function.
- Add an integration test for any new CLI command.
- Not reduce coverage of existing modules.

---

## Commit Message Convention

Use the format:

```
<type>(<scope>): <short summary>

<optional body>
```

### Types

| Type | Description |
|---|---|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `refactor` | Code restructuring without behavior change |
| `test` | Adding or fixing tests |
| `chore` | Build, CI, dependency updates |

### Examples

```
feat(port): add --kill flag to terminate port owner
fix(envcheck): handle empty .env files without panic
docs(readme): add installation section
test(snap): add unit test for snapshot serialization
chore(ci): add macos-14 to CI matrix
```

---

## Pull Request Process

1. Ensure all CI checks pass (format, lint, test on all platforms).
2. Fill out the PR description: what it does, why, and how to test.
3. Link any related issues.
4. Request review from a maintainer.
5. Address review feedback with fixup commits; the reviewer will squash-merge.

### PR Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --all` passes
- [ ] New behavior has tests
- [ ] Documentation updated if user-facing behavior changed
- [ ] No secrets in code or test fixtures

---

## Adding a New Command

Here is the step-by-step process for adding a new CLI subcommand (e.g., `devflow audit`):

### 1. Define the CLI variant

In `src/cli.rs`, add a variant to the `Command` enum:

```rust
#[derive(Debug, Subcommand)]
pub enum Command {
    // ... existing variants
    Audit(AuditArgs),
}

#[derive(Debug, Args)]
pub struct AuditArgs {
    #[arg(long)]
    pub verbose: bool,
}
```

### 2. Create the command module

Create `src/commands/audit.rs`:

```rust
use anyhow::Result;
use crate::cli::AuditArgs;

pub async fn run(args: AuditArgs) -> Result<()> {
    // Implementation
    Ok(())
}
```

### 3. Register the module

In `src/commands/mod.rs`:

```rust
pub mod audit;  // add this line

// In the run() function match:
Command::Audit(args) => audit::run(args).await,
```

### 4. Add tests

- Unit test in `src/commands/audit.rs` (if there's testable logic).
- Integration test in `tests/integration/cli_tests.rs`:

```rust
#[test]
fn audit_runs_successfully() {
    let td = tempfile::tempdir().expect("tempdir");
    cargo_bin_cmd!("devflow")
        .current_dir(td.path())
        .arg("audit")
        .assert()
        .success();
}
```

### 5. Update documentation

- Add the command to `docs/command-reference.md`.
- Add usage details to `docs/usage.md`.
- Update the commands table in `README.md`.

---

## Adding a New Utility Module

1. Create `src/utils/myutil.rs`.
2. Add `pub mod myutil;` to `src/utils/mod.rs`.
3. Add unit tests inside the file.
4. Document the module's purpose in `docs/api-reference.md`.

---

## Writing a Plugin

See [docs/plugin.md](docs/plugin.md) for the full plugin development guide. In brief:

1. Create an executable named `devflow-plugin-<name>` (any language).
2. Read JSON from stdin, write JSON to stdout.
3. Place it in `./plugins/` or install it in `PATH`.
4. Test with: `devflow plugin <name> --payload '{}'`

---

## Release Process

Releases are automated via GitHub Actions (see `.github/workflows/release.yml`):

1. Tag a commit: `git tag v0.2.0 && git push --tags`
2. CI builds binaries for Linux, macOS, Windows.
3. Artifacts are uploaded to the GitHub Release.
4. The Homebrew formula is regenerated.

For manual packaging, see [docs/deployment.md](docs/deployment.md).
