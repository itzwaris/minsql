#!/bin/bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

NUM_NODES="${1:-3}"

echo "==> Starting $NUM_NODES-node cluster"

for i in $(seq 1 $NUM_NODES); do
    PORT=$((5432 + i))
    DATA_DIR="./data/node-$i"
    
    mkdir -p "$DATA_DIR"
    
    echo "==> Starting node $i on port $PORT"
    
    target/release/minsql \
        --node-id "$i" \
        --data-dir "$DATA_DIR" \
        --port "$PORT" \
        --peers "localhost:5433,localhost:5434,localhost:5435" \
        > "$DATA_DIR/minsql.log" 2>&1 &
    
    echo $! > "$DATA_DIR/minsql.pid"
done

echo "==> Cluster started"
echo "    Node 1: localhost:5433"
echo "    Node 2: localhost:5434"
echo "    Node 3: localhost:5435"
