# Contributing

## Setup

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Standards

- Keep runtime local and deterministic.
- Avoid logging secrets.
- Add tests for behavior changes.
- Keep cross-platform compatibility.
