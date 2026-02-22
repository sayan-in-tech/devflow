# Usage Guide

## `devflow up`

Detects project language, checks local toolchain presence, validates `.env` against `.devflow.yaml`, and reports recommendations.

## `devflow port`

- `--free`: prints common free development ports.
- `--watch`: live monitor of common ports.
- `--port <N>`: show process diagnostics for a specific port.

## `devflow watch`

Watches filesystem changes, applies ignore globs from config, and runs language-specific tests.

## `devflow env`

- `doctor`: diagnose PATH and env schema issues.
- `fix`: create minimal `.env` when absent.
- `diff`: compare current env file with saved snapshot.

## `devflow logs`

Groups repeated error lines in `devflow.log` and tracks newly seen errors.

## `devflow deps`

Prints dependency metadata and offline risk hints for Python/Node/Rust projects.

## `devflow snap`

- `save`: save current process/env snapshot to `.devflow/snapshot.json`.
- `restore`: print snapshot items that would be restored.

## `devflow dash`

Opens terminal dashboard with CPU, memory, and process counts.

## `devflow init`

Creates `.devflow.yaml` template.
