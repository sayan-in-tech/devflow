# Plugin Development Guide

> How to write, test, and distribute plugins for devflow.

---

## Table of Contents

- [Overview](#overview)
- [How Plugins Work](#how-plugins-work)
- [Plugin Discovery](#plugin-discovery)
- [Protocol Specification](#protocol-specification)
- [Writing a Plugin](#writing-a-plugin)
  - [Python Example](#python-example)
  - [Node.js Example](#nodejs-example)
  - [Bash Example](#bash-example)
  - [Go Example](#go-example)
- [Testing Plugins](#testing-plugins)
- [Error Handling](#error-handling)
- [Security Considerations](#security-considerations)
- [Bundled Plugins](#bundled-plugins)
- [WASM Plugins (Planned)](#wasm-plugins-planned)

---

## Overview

`devflow` supports executable plugins that extend its functionality. Plugins:

- Can be written in **any programming language**
- Communicate via **JSON on stdin/stdout**
- Are resolved from **`PATH`** or **`./plugins/`**
- Run as **child processes** (not in-process)
- Receive structured input and return structured output

---

## How Plugins Work

```
devflow plugin <name> --payload '...'
    │
    ▼
1. Resolve executable: devflow-plugin-<name>
2. Spawn child process (stdin piped, stdout piped)
3. Write PluginRequest JSON to stdin
4. Wait for child to exit
5. Parse PluginResponse JSON from stdout
6. Pretty-print response
```

---

## Plugin Discovery

devflow resolves plugin executables in this order:

### 1. PATH Lookup

devflow prepends `devflow-plugin-` to the name (if not already present) and searches `PATH`:

```
devflow plugin foo  →  which("devflow-plugin-foo")
```

### 2. Local `./plugins/` Directory

If not found in `PATH`, checks the project-local `plugins/` directory:

```
devflow plugin foo  →  ./plugins/devflow-plugin-foo
```

### 3. Failure

If neither location has the executable:

```
Error: plugin not found: devflow-plugin-foo
```

### Naming Convention

| Plugin Name (CLI) | Executable Name |
|---|---|
| `infra-check` | `devflow-plugin-infra-check` |
| `devflow-plugin-lint` | `devflow-plugin-lint` |
| `my-tool` | `devflow-plugin-my-tool` |

The `devflow-plugin-` prefix is added automatically if not present.

---

## Protocol Specification

### Request (stdin)

devflow writes a JSON object to the plugin's stdin:

```json
{
  "command": "infra-check",
  "payload": {
    "key": "value"
  }
}
```

| Field | Type | Description |
|---|---|---|
| `command` | `string` | The plugin name as invoked |
| `payload` | `object` | The parsed `--payload` JSON, or `{}` if not supplied |

**Payload parsing**: If `--payload` is provided but is not valid JSON, it's wrapped as:

```json
{ "raw": "<original string>" }
```

### Response (stdout)

The plugin must write a JSON object to stdout:

```json
{
  "ok": true,
  "message": "human-readable status message",
  "data": {
    "key": "value"
  }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `ok` | `boolean` | Yes | Whether the plugin operation succeeded |
| `message` | `string` | Yes | Human-readable description of the result |
| `data` | `object` | Yes | Arbitrary structured data (can be `{}`) |

### Exit Code

- **Exit 0**: devflow parses stdout as `PluginResponse`.
- **Exit non-zero**: devflow returns an error: `plugin exited with status <N>`.

### Stderr

Plugin stderr is inherited by devflow's process. Any stderr output from the plugin will appear in the terminal. Use stderr for debug/error logging that shouldn't be part of the structured response.

---

## Writing a Plugin

### Python Example

`plugins/devflow-plugin-hello`:

```python
#!/usr/bin/env python3
import json
import sys

def main():
    raw = sys.stdin.read().strip() or "{}"
    req = json.loads(raw)

    name = req.get("payload", {}).get("name", "World")

    response = {
        "ok": True,
        "message": f"Hello, {name}!",
        "data": {
            "command": req.get("command"),
            "greeting": f"Hello, {name}!",
        },
    }
    sys.stdout.write(json.dumps(response))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
```

```bash
chmod +x plugins/devflow-plugin-hello
devflow plugin hello --payload '{"name": "Alice"}'
```

### Node.js Example

`plugins/devflow-plugin-check-deps`:

```javascript
#!/usr/bin/env node
const fs = require('fs');

let input = '';
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
    const req = JSON.parse(input || '{}');

    let depCount = 0;
    try {
        const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
        depCount = Object.keys(pkg.dependencies || {}).length;
    } catch (e) {
        // no package.json
    }

    const response = {
        ok: true,
        message: `Found ${depCount} dependencies`,
        data: {
            command: req.command,
            dependency_count: depCount,
        },
    };
    process.stdout.write(JSON.stringify(response));
});
```

### Bash Example

`plugins/devflow-plugin-disk-check`:

```bash
#!/usr/bin/env bash
read -r INPUT
USAGE=$(df -h . | tail -1 | awk '{print $5}')

cat <<EOF
{
  "ok": true,
  "message": "Disk usage: ${USAGE}",
  "data": {
    "usage_percent": "${USAGE}"
  }
}
EOF
```

### Go Example

`plugins/devflow-plugin-health/main.go`:

```go
package main

import (
    "encoding/json"
    "fmt"
    "os"
    "runtime"
)

type Request struct {
    Command string                 `json:"command"`
    Payload map[string]interface{} `json:"payload"`
}

type Response struct {
    OK      bool                   `json:"ok"`
    Message string                 `json:"message"`
    Data    map[string]interface{} `json:"data"`
}

func main() {
    var req Request
    json.NewDecoder(os.Stdin).Decode(&req)

    resp := Response{
        OK:      true,
        Message: "System healthy",
        Data: map[string]interface{}{
            "os":      runtime.GOOS,
            "arch":    runtime.GOARCH,
            "cpus":    runtime.NumCPU(),
            "command": req.Command,
        },
    }

    json.NewEncoder(os.Stdout).Encode(resp)
}
```

Build and install:

```bash
cd plugins/devflow-plugin-health
go build -o ../devflow-plugin-health
devflow plugin health
```

---

## Testing Plugins

### Manual Testing

Pipe JSON directly to the plugin:

```bash
echo '{"command":"test","payload":{}}' | python plugins/devflow-plugin-infra-check.py
```

### Testing via devflow

```bash
devflow plugin infra-check
devflow plugin infra-check --payload '{"region": "us-east-1"}'
```

### Automated Testing

Write tests in the plugin's native language:

```python
# test_plugin.py
import subprocess
import json

def test_infra_check():
    result = subprocess.run(
        ["python", "plugins/devflow-plugin-infra-check.py"],
        input='{"command":"test","payload":{}}',
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0
    resp = json.loads(result.stdout)
    assert "ok" in resp
    assert "message" in resp
    assert "data" in resp
```

---

## Error Handling

### Plugin Not Found

```
Error: plugin not found: devflow-plugin-nonexistent
```

Ensure the plugin exists in `PATH` or `./plugins/` and is executable.

### Invalid JSON Response

```
Error: plugin produced invalid JSON
```

The plugin's stdout must be valid JSON matching the `PluginResponse` schema. Ensure no other output (debug prints, warnings) is mixed into stdout. Use stderr for non-structured output.

### Non-Zero Exit Code

```
Error: plugin exited with status 1
```

The plugin process exited with an error. Check the plugin's stderr for details.

### Timeout

There is currently no built-in timeout for plugin execution. If a plugin hangs, press `Ctrl+C` to terminate devflow and the child plugin process.

---

## Security Considerations

1. **Plugins run with full process privileges**: They have the same permissions as the `devflow` process. Only use trusted plugins.
2. **No sandboxing**: Plugins can access the filesystem, network, and environment.
3. **Payloads may contain sensitive data**: Be careful not to log or expose the `--payload` content in plugin output if it contains secrets.
4. **Plugin discovery via PATH**: A malicious plugin in `PATH` named `devflow-plugin-<name>` would be executed. Be aware of your `PATH` contents.

---

## Bundled Plugins

### `infra-check`

**File**: `plugins/devflow-plugin-infra-check.py`
**Language**: Python 3
**Purpose**: Verify infrastructure credentials are present.

**Behavior**:
1. Reads JSON request from stdin.
2. Checks for `AWS_PROFILE` and `KUBECONFIG` environment variables.
3. Returns success if both are set, failure with a list of missing credentials otherwise.

**Example**:

```bash
$ devflow plugin infra-check
{
  "ok": false,
  "message": "missing infra credentials",
  "data": {
    "command": "infra-check",
    "missing": ["AWS_PROFILE", "KUBECONFIG"]
  }
}
```

---

## WASM Plugins (Planned)

Plugin names ending in `.wasm` are recognized by devflow but currently return an error:

```
Error: WASM plugin runtime not enabled in this build
```

When implemented, WASM plugins will:
- Run in a sandboxed WebAssembly runtime (e.g., wasmtime)
- Have a defined capability model (filesystem access, env var access, etc.)
- Use the same JSON stdin/stdout protocol
- Provide stronger security guarantees than executable plugins

Use executable plugins for now.
