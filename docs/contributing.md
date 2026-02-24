-- Active: 1754307105999@@127.0.0.1@3306
# Contributing (Quick Reference)

> Quick-start guide for contributors. For the complete contributing guide, see [CONTRIBUTING.md](../CONTRIBUTING.md) in the project root.

---

## Setup

```bash
git clone https://github.com/example/devflow.git
cd devflow
cargo build
```

## Check Before Submitting

```bash
cargo fmt --all -- --check                                # formatting
cargo clippy --all-targets --all-features -- -D warnings  # linting
cargo test --all                                          # all tests
```

All three must pass.

## Standards

- **Local-first**: No network calls, no telemetry.
- **Deterministic**: Same inputs → same outputs.
- **Secret-safe**: Never log or print raw secrets. Use `sanitize::redact()`.
- **Cross-platform**: Must work on Linux, macOS, and Windows.
- **Tested**: Add unit tests for utility functions, integration tests for CLI commands.

## Adding a New Command

1. Add a variant to `Command` in `src/cli.rs`
2. Create `src/commands/<name>.rs`
3. Register in `src/commands/mod.rs`
4. Add tests
5. Update docs

## Commit Format

```
<type>(<scope>): <summary>
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

## Full Guide

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Detailed development workflow
- PR process and checklist
- How to add utility modules
- How to write plugins
- Release process
