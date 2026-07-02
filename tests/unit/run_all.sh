#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Running all unit tests ==="
echo ""

echo "--- Users Lambda ---"
cd "$PROJECT_ROOT/src/functions/users"
cargo test
echo ""

echo "--- Authorizer Lambda ---"
cd "$PROJECT_ROOT/src/functions/authorizer"
cargo test
echo ""

echo "=== All unit tests passed ==="
