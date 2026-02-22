#!/usr/bin/env python3
import json
import os
import sys


def main() -> int:
    raw = sys.stdin.read().strip() or "{}"
    req = json.loads(raw)
    required = ["AWS_PROFILE", "KUBECONFIG"]
    missing = [k for k in required if not os.environ.get(k)]
    ok = len(missing) == 0
    response = {
        "ok": ok,
        "message": "infra credentials healthy" if ok else "missing infra credentials",
        "data": {
            "command": req.get("command"),
            "missing": missing,
        },
    }
    sys.stdout.write(json.dumps(response))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
