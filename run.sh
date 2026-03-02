#!/usr/bin/env bash
# run.sh — build and launch NyxForge (node + Flutter UI)
#
# Usage:
#   ./run.sh                     # default: testnet, 1 mining thread, no auto-mine
#   ./run.sh --mine               # start mining immediately
#   ./run.sh --mine --threads 4  # mine with 4 CPU threads
#   ./run.sh --mainnet            # mainnet (when live)
#
# Stops everything cleanly on Ctrl-C.

set -euo pipefail

FLUTTER="${FLUTTER:-$HOME/development/flutter/bin/flutter}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NODE_ARGS=(--testnet)
MINE=false
THREADS=1
OPEN_BROWSER=true

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
    case "$1" in
        --mine)       MINE=true ;;
        --mainnet)    NODE_ARGS=(${NODE_ARGS[@]/--testnet/}) ;;
        --threads)    THREADS="$2"; shift ;;
        --no-browser) OPEN_BROWSER=false ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
    shift
done

[[ "$MINE" == true ]] && NODE_ARGS+=(--mine-on-start --mine-threads "$THREADS")

# ---------------------------------------------------------------------------
# Verify tools
# ---------------------------------------------------------------------------
if ! command -v cargo &>/dev/null; then
    echo "Error: cargo not found. Install Rust: https://rustup.rs" >&2
    exit 1
fi
if [[ ! -x "$FLUTTER" ]]; then
    echo "Error: flutter not found at $FLUTTER" >&2
    echo "Set FLUTTER=/path/to/flutter/bin/flutter or install Flutter." >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Build node (skipped if already up to date)
# ---------------------------------------------------------------------------
echo "==> Building nyxforge-node..."
cd "$SCRIPT_DIR"
cargo build -p nyxforge-node 2>&1

# ---------------------------------------------------------------------------
# Start node in background
# ---------------------------------------------------------------------------
NODE_BIN="$SCRIPT_DIR/target/debug/nyxforge-node"
NODE_LOG="$SCRIPT_DIR/node.log"

echo "==> Starting node (logs → $NODE_LOG)"
"$NODE_BIN" "${NODE_ARGS[@]}" > "$NODE_LOG" 2>&1 &
NODE_PID=$!

# Wait for RPC to be ready (up to 10 s)
echo -n "    Waiting for RPC..."
for i in $(seq 1 20); do
    if curl -sf -X POST http://127.0.0.1:8888/rpc \
            -H 'Content-Type: application/json' \
            -d '{"method":"status","params":{}}' &>/dev/null; then
        echo " ready."
        break
    fi
    sleep 0.5
    echo -n "."
    if [[ $i -eq 20 ]]; then
        echo " timed out. Check $NODE_LOG"
    fi
done

# ---------------------------------------------------------------------------
# Clean up node + Flutter on exit
# ---------------------------------------------------------------------------
cleanup() {
    echo
    echo "==> Shutting down..."
    kill "$NODE_PID" 2>/dev/null || true
    # Flutter run exits when its terminal input closes; SIGINT propagates.
}
trap cleanup INT TERM EXIT

# ---------------------------------------------------------------------------
# Open browser (Chrome) after a short delay so Flutter has time to compile
# ---------------------------------------------------------------------------
if [[ "$OPEN_BROWSER" == true ]]; then
    (sleep 8 && open -a "Google Chrome" http://localhost:8080) &
fi

# ---------------------------------------------------------------------------
# Start Flutter (foreground — Ctrl-C here stops everything via trap)
# ---------------------------------------------------------------------------
echo "==> Starting Flutter UI..."
cd "$SCRIPT_DIR/ui"
"$FLUTTER" run -d chrome --web-port 8080
