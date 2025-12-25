#!/bin/bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

BUILD_TYPE="${1:-release}"

echo "==> Building minsql ($BUILD_TYPE)"

if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release
    echo "==> Binary: target/release/minsql"
elif [ "$BUILD_TYPE" = "debug" ]; then
    cargo build
    echo "==> Binary: target/debug/minsql"
else
    echo "Error: Unknown build type '$BUILD_TYPE'"
    echo "Usage: $0 [release|debug]"
    exit 1
fi

echo "==> Running tests"
cargo test

echo "==> Build complete"
