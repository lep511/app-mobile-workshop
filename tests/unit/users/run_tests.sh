#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CRATE_DIR="$PROJECT_ROOT/src/functions/users"

echo "=== Users Lambda — Unit Tests ==="
cd "$CRATE_DIR"
cargo test "$@"
