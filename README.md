# devflow

`devflow` is a local-first, cross-platform developer workflow CLI written in Rust.

## Quick start

```bash
cargo run -- init
cargo run -- up
cargo run -- env doctor
```

## Commands

- `devflow up`
- `devflow port [--port N|--free|--watch]`
- `devflow watch`
- `devflow env <doctor|fix|diff>`
- `devflow logs`
- `devflow deps`
- `devflow snap <save|restore>`
- `devflow dash`
- `devflow init`
- `devflow plugin <name> [--payload JSON]`

See [docs/usage.md](docs/usage.md) for full details.
