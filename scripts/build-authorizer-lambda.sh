#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LAMBDA_DIR="$PROJECT_ROOT/src/functions/authorizer"
OUTPUT_DIR="$PROJECT_ROOT/infrastructure/dist"

mkdir -p "$OUTPUT_DIR"

echo "Building authorizer-lambda with cargo-lambda..."
cd "$LAMBDA_DIR"
cargo lambda build --release --arm64

echo "Packaging bootstrap binary..."
cd "$LAMBDA_DIR/target/lambda/authorizer-lambda"
zip "$OUTPUT_DIR/authorizer-lambda.zip" bootstrap

echo "Done: $OUTPUT_DIR/authorizer-lambda.zip"
