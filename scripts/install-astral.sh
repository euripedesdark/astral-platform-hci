#!/usr/bin/env bash
# Rust/React deployment; shared configuration and databases are preserved.
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "$ROOT/deploy_platform.py" "$@"
