#!/usr/bin/env bash
set -euo pipefail

cargo build --release
mkdir -p dist
cp target/release/devflow dist/ || true

echo "Create .deb/.rpm with nfpm or cargo-deb in CI"
echo "Create MSI/EXE installer with WiX in CI"
