# Troubleshooting Guide

> Common issues, error messages, and solutions for devflow.

---

## Table of Contents

- [Installation Issues](#installation-issues)
- [Build Issues](#build-issues)
- [Command-Specific Issues](#command-specific-issues)
  - [devflow init](#devflow-init)
  - [devflow up](#devflow-up)
  - [devflow env](#devflow-env)
  - [devflow port](#devflow-port)
  - [devflow watch](#devflow-watch)
  - [devflow logs](#devflow-logs)
  - [devflow snap](#devflow-snap)
  - [devflow dash](#devflow-dash)
  - [devflow plugin](#devflow-plugin)
- [Configuration Issues](#configuration-issues)
- [Cross-Platform Issues](#cross-platform-issues)
- [Performance Issues](#performance-issues)
- [Getting Help](#getting-help)

---

## Installation Issues

### Rust/Cargo not found

**Error**: `cargo: command not found` or `rustc: command not found`

**Solution**: Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Windows, download and run `rustup-init.exe` from https://rustup.rs/.

After installation, restart your terminal or run:

```bash
source ~/.cargo/env
```

### Compilation errors on Windows

**Error**: Linker errors mentioning `link.exe`

**Solution**: Install the [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "C++ Build Tools" workload, which includes MSVC and the Windows SDK.

### Compilation errors related to `sysinfo`

**Error**: Errors in `sysinfo` or `ntapi` crate compilation

**Solution**: Ensure you're on a recent version of Rust:

```bash
rustup update stable
```

The `sysinfo` crate requires Rust 1.70+.

---

## Build Issues

### `cargo build` fails with dependency resolution errors

**Solution**: Clean the build cache and retry:

```bash
cargo clean
cargo build
```

If the issue persists, delete `Cargo.lock` and rebuild:

```bash
rm Cargo.lock
cargo build
```

### Build times are very slow

**Possible causes**:
- First build compiles all dependencies
- Debug builds include full debug info

**Solutions**:
- Use `cargo build` (debug) for development — it's faster than `cargo build --release`
- Ensure `Swatinem/rust-cache@v2` is used in CI (already configured)
- Consider using [sccache](https://github.com/mozilla/sccache) for local builds

---

## Command-Specific Issues

### devflow init

#### `.devflow.yaml already exists`

**Cause**: `devflow init` is idempotent. It won't overwrite an existing config.

**Solution**: If you want to regenerate, delete the existing file first:

```bash
rm .devflow.yaml
devflow init
```

---

### devflow up

#### `recommendation: run devflow init to create .devflow.yaml`

**Cause**: No `.devflow.yaml` found in the current directory.

**Solution**: Run `devflow init` to create one, then edit it to match your project.

#### `toolchain: missing (python)`

**Cause**: The detected project language's toolchain is not installed or not in `PATH`.

**Solution**: Install the required toolchain:

| Language | Install |
|---|---|
| Python | `pyenv install 3.x` or system package manager |
| Node | `nvm install 18` or download from nodejs.org |
| Rust | `rustup install stable` |
| Go | `go install` or download from golang.org |

Then ensure the binary is in your `PATH`.

#### `env: N issues`

**Cause**: Your `.env` file doesn't match the schema in `.devflow.yaml`.

**Solution**:
1. Run `devflow env doctor` for detailed diagnostics.
2. Run `devflow env fix` to create a minimal `.env` if missing.
3. Manually add the required environment variables.

---

### devflow env

#### `env doctor: Python not found in PATH`

**Cause**: `python` and `python3` are not in `PATH`. This is a general system check, not specific to your project.

**Solution**: Install Python or, if it's not relevant to your project, this warning can be ignored.

#### `env fix: .env already exists; no changes`

**Cause**: `.env` already exists. `env fix` only creates the file; it doesn't modify existing files.

**Solution**: Manually edit `.env` to add missing variables identified by `env doctor`.

#### `env diff: saved first env snapshot`

**Cause**: This is the first run of `env diff`. It saves the current `.env` as a baseline.

**Solution**: This is normal. Run `env diff` again after making changes to see the diff.

---

### devflow port

#### `No process found for port 3000`

**Cause**: No running process is using port 3000 (or the heuristic detection didn't match).

**Notes**: Port detection works by scanning process names and command-line arguments for the port number. It may miss processes that don't include the port number in their command line.

**Alternative**: Use OS-native tools for precise port inspection:

```bash
# Linux/macOS
lsof -i :3000
netstat -tlnp | grep 3000

# Windows
netstat -ano | findstr 3000
```

#### `port --watch` doesn't stop

**Cause**: `port --watch` runs in an infinite loop by design.

**Solution**: Press `Ctrl+C` to stop.

---

### devflow watch

#### `watching for changes...` but tests don't run

**Possible causes**:
1. Changes are in files matching `ignore_globs` patterns.
2. The test command for your language is not installed.

**Solutions**:
1. Check `ignore_globs` in `.devflow.yaml`. The patterns `target/**` and `node_modules/**` are included by default.
2. Ensure the test runner is installed:
   - Python: `pip install pytest`
   - Node: `npm install --save-dev jest`
   - Rust: comes with `cargo`
   - Go: comes with `go`

#### Test command fails

**Cause**: The default test commands may not match your project setup.

**Test commands by language**:
| Language | Default Test Command |
|---|---|
| Python | `pytest -q` |
| Node | `npx jest --passWithNoTests` |
| Rust | `cargo test` |
| Go | `go test ./...` |

**Solution**: If your project uses a different test runner (e.g., `mocha`, `vitest`, `unittest`), the current `watch` command doesn't support custom test commands via config (the `test_command` config field is not yet used by `watch`). This is a known limitation.

---

### devflow logs

#### `No devflow.log found`

**Cause**: devflow reads `devflow.log` from the current directory. This file must exist and contain log output.

**Solution**: If your application logs to a different file, you'll need to symlink or copy it:

```bash
ln -s path/to/your/app.log devflow.log
```

---

### devflow snap

#### `Error: No such file or directory` on `snap restore`

**Cause**: No snapshot has been saved yet.

**Solution**: Run `devflow snap save` first.

#### Snapshot doesn't capture my processes

**Cause**: `snap save` only captures processes whose command line contains the current directory path or whose name contains "cargo".

**Solution**: Ensure your development processes are started from within the project directory.

---

### devflow dash

#### Terminal garbled after crash

**Cause**: If `devflow dash` exits abnormally (e.g., panic, `Ctrl+C` during startup), the terminal may be left in raw mode.

**Solution**: Run `reset` (Linux/macOS) or close and reopen the terminal (Windows).

```bash
reset
```

#### Dashboard shows 0% CPU

**Cause**: On some systems, the first CPU measurement is always 0%. `sysinfo` needs at least two measurements to compute usage.

**Solution**: Wait a few seconds. The dashboard should start showing real CPU values after 1-2 refresh cycles.

---

### devflow plugin

#### `plugin not found: devflow-plugin-<name>`

**Cause**: The plugin executable cannot be found in `PATH` or in `./plugins/`.

**Solutions**:
1. Ensure the plugin file exists in `./plugins/devflow-plugin-<name>` or is installed globally.
2. On Linux/macOS, ensure the plugin is executable:
   ```bash
   chmod +x plugins/devflow-plugin-<name>
   ```
3. On Windows, ensure the plugin has a recognized extension (`.exe`, `.bat`, `.cmd`, `.py`, etc.) or has a shebang and Python/Node in PATH.

#### `plugin produced invalid JSON`

**Cause**: The plugin's stdout does not contain valid JSON matching the expected response schema.

**Expected response format**:
```json
{
  "ok": true,
  "message": "description",
  "data": {}
}
```

**Debug**: Run the plugin directly to inspect its output:

```bash
echo '{"command":"test","payload":{}}' | python plugins/devflow-plugin-<name>
```

#### `plugin exited with status 1`

**Cause**: The plugin process returned a non-zero exit code.

**Solution**: Check the plugin's stderr output. Run the plugin directly to debug.

#### `WASM plugin runtime not enabled in this build`

**Cause**: WASM plugins are recognized but not yet implemented.

**Solution**: Use executable plugins instead. WASM support is planned for a future release.

---

## Configuration Issues

### Invalid YAML in `.devflow.yaml`

**Error**: `invalid .devflow.yaml`

**Solution**: Validate your YAML syntax. Common mistakes:
- Incorrect indentation (YAML uses spaces, not tabs)
- Missing colons after keys
- Incorrect list formatting

Use a YAML validator or check against the [example config](../examples/sample.devflow.yaml).

### Unknown fields in config

**Behavior**: Unknown fields are silently ignored (serde default behavior).

**Note**: Typos in field names won't cause errors but the values won't be used. Double-check field names against the [configuration reference](configuration.md).

---

## Cross-Platform Issues

### `kill` command doesn't work on Windows

**Cause**: `devflow port` suggests both Unix and Windows kill commands. On Windows, use the `taskkill` variant:

```powershell
taskkill /PID <pid>       # graceful
taskkill /F /PID <pid>    # force
```

### File watching doesn't detect changes

**Possible causes**:
- On Linux: `inotify` watch limit exceeded
- On Windows: Files in network shares may not trigger events
- On macOS: Some file operations may be batched by FSEvents

**Linux fix**: Increase the inotify watch limit:

```bash
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### Path issues on Windows

**Cause**: Some operations may have issues with long paths on Windows.

**Solution**: Enable long path support in Windows:

```powershell
# Run as Administrator
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

---

## Performance Issues

### `devflow up` is slow

**Cause**: `which::which()` searches the entire `PATH` for each tool, and `sysinfo` can be slow to initialize.

**Mitigation**: This is usually under 1 second. If it's consistently slow, check if your `PATH` contains many directories or network paths.

### `devflow snap save` is slow

**Cause**: `System::new_all()` enumerates all running processes on the system.

**Mitigation**: This is inherent to the `sysinfo` API. The operation typically takes 1-3 seconds depending on the number of running processes.

---

## Getting Help

1. **Check this guide** for your specific error message.
2. **Run with debug logging**: `RUST_LOG=debug devflow <command>`
3. **Open an issue** on GitHub with:
   - The exact command you ran
   - The full error output
   - Your OS and Rust version (`rustc --version`)
   - The contents of your `.devflow.yaml` (if relevant)
4. **Check existing issues** on the GitHub repository.
