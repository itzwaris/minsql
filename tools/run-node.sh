#!/bin/bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

NODE_ID="${1:-1}"
DATA_DIR="${2:-./data/node-$NODE_ID}"
PORT="${3:-5433}"

mkdir -p "$DATA_DIR"

echo "==> Starting minsql node"
echo "    Node ID: $NODE_ID"
echo "    Data directory: $DATA_DIR"
echo "    Port: $PORT"

exec target/release/minsql \
    --node-id "$NODE_ID" \
    --data-dir "$DATA_DIR" \
    --port "$PORT"
