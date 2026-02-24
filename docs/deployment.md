# Deployment, Build & Packaging Guide

> How to build, package, and distribute devflow across all supported platforms.

---

## Table of Contents

- [Build from Source](#build-from-source)
- [Release Process](#release-process)
- [GitHub Actions CI/CD](#github-actions-cicd)
- [Platform-Specific Packaging](#platform-specific-packaging)
  - [Linux (.deb / .rpm)](#linux-deb--rpm)
  - [macOS (Homebrew)](#macos-homebrew)
  - [Windows (Installer / Winget)](#windows-installer--winget)
- [Manual Distribution](#manual-distribution)
- [Cross-Compilation](#cross-compilation)
- [Version Management](#version-management)
- [Release Checklist](#release-checklist)

---

## Build from Source

### Debug Build

```bash
cargo build
# Binary: target/debug/devflow (or devflow.exe on Windows)
```

### Release Build

```bash
cargo build --release
# Binary: target/release/devflow (or devflow.exe on Windows)
```

### Install Locally

```bash
cargo install --path .
# Installs to ~/.cargo/bin/devflow
```

### Build Script

The `scripts/build-packages.sh` script automates the release build:

```bash
bash scripts/build-packages.sh
```

This runs `cargo build --release` and copies the binary to `dist/`.

---

## Release Process

### 1. Update Version

Edit `Cargo.toml`:

```toml
[package]
version = "0.2.0"
```

Also update version references in:
- `packaging/nfpm.yaml`
- `packaging/windows/installer.iss`
- `packaging/winget/devflow.yaml`
- `scripts/generate-homebrew-formula.sh`

### 2. Commit and Tag

```bash
git add -A
git commit -m "chore: release v0.2.0"
git tag v0.2.0
git push origin main --tags
```

### 3. CI Handles the Rest

Pushing the `v*` tag triggers `.github/workflows/release.yml`, which:

1. Builds release binaries for Linux, macOS, and Windows.
2. Uploads build artifacts to the GitHub Release.
3. Generates the Homebrew formula.

---

## GitHub Actions CI/CD

### CI Pipeline (`.github/workflows/ci.yml`)

**Triggers**: Every push, every pull request.

**Matrix**:
| OS | Toolchain |
|---|---|
| `ubuntu-latest` | stable |
| `macos-latest` | stable |
| `windows-latest` | stable |

**Steps**:

```yaml
- uses: actions/checkout@v4
- uses: dtolnay/rust-toolchain@stable
- uses: Swatinem/rust-cache@v2        # dependency caching
- cargo fmt --all -- --check           # formatting
- cargo clippy --all-targets -- -D warnings  # linting
- cargo test --all                     # testing
```

### Release Pipeline (`.github/workflows/release.yml`)

**Triggers**: Push of `v*` tag, manual `workflow_dispatch`.

**Build Matrix**:

| OS | Target Triple | Artifact Name |
|---|---|---|
| `ubuntu-latest` | `x86_64-unknown-linux-gnu` | `devflow-linux-x86_64` |
| `macos-latest` | `x86_64-apple-darwin` | `devflow-macos-x86_64` |
| `windows-latest` | `x86_64-pc-windows-msvc` | `devflow-windows-x86_64.exe` |

**Steps**:

```yaml
# Build job (per-platform)
- cargo build --release --target ${{ matrix.target }}
- actions/upload-artifact@v4

# Publish job (ubuntu-latest)
- actions/download-artifact@v4
- bash scripts/generate-homebrew-formula.sh
```

---

## Platform-Specific Packaging

### Linux (.deb / .rpm)

**Tool**: [nfpm](https://nfpm.goreleaser.com/)

**Config**: `packaging/nfpm.yaml`

```yaml
name: devflow
arch: amd64
platform: linux
version: 0.1.0
section: default
priority: optional
maintainer: Devflow Maintainers <maintainers@example.com>
description: |
  Developer workflow automation CLI.
license: MIT
contents:
  - src: ./target/release/devflow
    dst: /usr/local/bin/devflow
```

**Build Commands**:

```bash
# Install nfpm
go install github.com/goreleaser/nfpm/v2/cmd/nfpm@latest

# Build .deb
nfpm package --config packaging/nfpm.yaml --target devflow.deb --packager deb

# Build .rpm
nfpm package --config packaging/nfpm.yaml --target devflow.rpm --packager rpm
```

**Installation**:

```bash
# Debian/Ubuntu
sudo dpkg -i devflow.deb

# RHEL/Fedora
sudo rpm -i devflow.rpm
```

---

### macOS (Homebrew)

**Generator**: `scripts/generate-homebrew-formula.sh`

This script generates `packaging/homebrew/devflow.rb`:

```ruby
class Devflow < Formula
  desc "Developer workflow automation"
  homepage "https://github.com/example/devflow"
  url "https://github.com/example/devflow/releases/download/v0.1.0/devflow-macos-x86_64.tar.gz"
  sha256 "REPLACE_ME"
  version "0.1.0"
  def install
    bin.install "devflow"
  end
end
```

**Usage**:

```bash
# Generate the formula
bash scripts/generate-homebrew-formula.sh

# Tap and install (after publishing the formula)
brew tap example/devflow
brew install devflow
```

**Note**: Replace `REPLACE_ME` with the actual SHA256 of the release tarball.

---

### Windows (Installer / Winget)

#### InnoSetup Installer

**Config**: `packaging/windows/installer.iss`

```ini
[Setup]
AppName=devflow
AppVersion=0.1.0
DefaultDirName={pf}\devflow

[Files]
Source: "..\..\target\release\devflow.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\devflow"; Filename: "{app}\devflow.exe"
```

**Build**:

1. Install [InnoSetup](https://jrsoftware.org/isinfo.php).
2. Open `packaging/windows/installer.iss` in InnoSetup.
3. Compile to generate `devflow-setup.exe`.

Or from command line:

```powershell
iscc packaging\windows\installer.iss
```

#### Winget Package

**Config**: `packaging/winget/devflow.yaml`

```yaml
PackageIdentifier: Devflow.Devflow
PackageVersion: 0.1.0
PackageName: devflow
Publisher: Devflow
License: MIT
ShortDescription: Developer workflow automation CLI
Installers:
  - Architecture: x64
    InstallerType: exe
    InstallerUrl: https://github.com/example/devflow/releases/download/v0.1.0/devflow-windows-x86_64.exe
    InstallerSha256: REPLACE_ME
```

**Publishing**: Submit to the [winget-pkgs](https://github.com/microsoft/winget-pkgs) repository.

**Installation**:

```powershell
winget install Devflow.Devflow
```

---

## Manual Distribution

For environments without package managers:

1. Download the binary for your platform from the releases page.
2. Make it executable (Linux/macOS):
   ```bash
   chmod +x devflow-linux-x86_64
   ```
3. Move to a directory in your `PATH`:
   ```bash
   sudo mv devflow-linux-x86_64 /usr/local/bin/devflow
   ```

On Windows, place `devflow.exe` anywhere and add that directory to your `PATH` environment variable.

---

## Cross-Compilation

To build for a different target from your development machine:

```bash
# Add the target
rustup target add x86_64-unknown-linux-gnu

# Build for that target
cargo build --release --target x86_64-unknown-linux-gnu
```

Common targets:

| Target | Platform |
|---|---|
| `x86_64-unknown-linux-gnu` | Linux x86_64 |
| `x86_64-apple-darwin` | macOS x86_64 |
| `aarch64-apple-darwin` | macOS Apple Silicon |
| `x86_64-pc-windows-msvc` | Windows x86_64 |
| `aarch64-unknown-linux-gnu` | Linux ARM64 |

**Note**: Cross-compilation may require platform-specific linkers and system libraries.

---

## Version Management

The version is defined in `Cargo.toml` under `[package].version`. This is the single source of truth.

Files that reference the version and must be updated for each release:

| File | Field |
|---|---|
| `Cargo.toml` | `version = "X.Y.Z"` |
| `packaging/nfpm.yaml` | `version: X.Y.Z` |
| `packaging/windows/installer.iss` | `AppVersion=X.Y.Z` |
| `packaging/winget/devflow.yaml` | `PackageVersion: X.Y.Z` |
| `scripts/generate-homebrew-formula.sh` | URL contains `vX.Y.Z` |

---

## Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update version in all packaging configs (see [Version Management](#version-management))
- [ ] Run full test suite: `cargo test --all`
- [ ] Run lints: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Update `CHANGELOG.md` (if maintained)
- [ ] Commit: `git commit -m "chore: release vX.Y.Z"`
- [ ] Tag: `git tag vX.Y.Z`
- [ ] Push: `git push origin main --tags`
- [ ] Verify CI release pipeline completes
- [ ] Download artifacts and verify they work
- [ ] Update Homebrew formula SHA256
- [ ] Submit winget package update (if applicable)
- [ ] Announce the release
