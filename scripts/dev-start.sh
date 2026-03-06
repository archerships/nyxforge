#!/usr/bin/env bash
# dev-start.sh -- Start the NyxForge development environment
#
# Usage:
#   ./scripts/dev-start.sh           # start node + seed + Flutter UI
#   ./scripts/dev-start.sh --no-ui   # start node + seed only (headless)
#   ./scripts/dev-start.sh --stop    # kill node and Flutter dev server
#
# The node runs in the background; Flutter opens Chrome on port 8080.
# Re-running is safe: existing processes are killed and restarted.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
UI_DIR="${PROJECT_ROOT}/ui"
NODE_BIN="${PROJECT_ROOT}/target/release/nyxforge-node"
DATA_DIR="${PROJECT_ROOT}/node_data_demo"
NODE_LOG="${PROJECT_ROOT}/node.log"
FLUTTER_LOG="${PROJECT_ROOT}/flutter.log"
FLUTTER_BIN="${HOME}/development/flutter/bin/flutter"
NODE_PORT=8888
UI_PORT=8080

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[dev]${RESET}  $*"; }
success() { echo -e "${GREEN}[dev]${RESET}  $*"; }
warn()    { echo -e "${YELLOW}[dev]${RESET}  $*"; }
die()     { echo -e "${RED}[dev] ERROR:${RESET} $*" >&2; exit 1; }

kill_port() {
    local port="$1" label="$2"
    local pid
    pid=$(lsof -ti :"${port}" 2>/dev/null || true)
    if [[ -n "${pid}" ]]; then
        kill "${pid}" 2>/dev/null || true
        # Wait up to 3 seconds for the port to free
        local i=0
        while lsof -ti :"${port}" &>/dev/null && (( i < 6 )); do
            sleep 0.5; (( i++ ))
        done
        info "Stopped ${label} (was PID ${pid})"
    fi
}

# -- --stop -------------------------------------------------------------------
if [[ "${1:-}" == "--stop" ]]; then
    kill_port "${NODE_PORT}" "nyxforge-node"
    kill_port "${UI_PORT}"   "flutter dev server"
    success "All dev processes stopped."
    exit 0
fi

START_UI=true
for arg in "$@"; do
    case "${arg}" in
        --no-ui) START_UI=false ;;
        *) die "Unknown argument: ${arg}" ;;
    esac
done

# -- Kill stale processes -----------------------------------------------------
info "Stopping any existing dev processes..."
kill_port "${NODE_PORT}" "nyxforge-node"
kill_port "${UI_PORT}"   "flutter dev server"

# -- Build node if needed -----------------------------------------------------
NODE_SRC_DIR="${PROJECT_ROOT}/crates/nyxforge-node"
if [[ ! -f "${NODE_BIN}" ]]; then
    info "Building nyxforge-node (release)..."
    cargo build -p nyxforge-node --release --manifest-path "${PROJECT_ROOT}/Cargo.toml"
    success "Build complete."
elif [[ -n "$(find "${NODE_SRC_DIR}/src" -name '*.rs' -newer "${NODE_BIN}" 2>/dev/null)" ]]; then
    info "Source changed — rebuilding nyxforge-node (release)..."
    cargo build -p nyxforge-node --release --manifest-path "${PROJECT_ROOT}/Cargo.toml"
    success "Build complete."
else
    info "nyxforge-node binary is up to date."
fi

# -- Start node ---------------------------------------------------------------
mkdir -p "${DATA_DIR}"
info "Starting nyxforge-node on port ${NODE_PORT}..."
nohup "${NODE_BIN}" \
    --allow-unverifiable \
    --data-dir "${DATA_DIR}" \
    > "${NODE_LOG}" 2>&1 &
NODE_PID=$!

# Wait for node to accept connections (up to 10 s)
READY=false
for i in $(seq 1 20); do
    if curl -sf -X POST "http://127.0.0.1:${NODE_PORT}/rpc" \
        -H 'Content-Type: application/json' \
        -d '{"method":"status","params":{}}' &>/dev/null; then
        READY=true
        break
    fi
    sleep 0.5
done

if [[ "${READY}" != "true" ]]; then
    die "Node did not start within 10 seconds. Check ${NODE_LOG}"
fi
success "Node is up (PID ${NODE_PID})"

# -- Seed demo bonds ----------------------------------------------------------
info "Seeding demo bonds..."
python3 "${SCRIPT_DIR}/seed-demo-bonds.py"

# -- Start Flutter ------------------------------------------------------------
if [[ "${START_UI}" == "true" ]]; then
    if [[ ! -f "${FLUTTER_BIN}" ]]; then
        die "Flutter not found at ${FLUTTER_BIN}. Run ./scripts/setup-dev.sh --flutter"
    fi

    info "Starting Flutter dev server on port ${UI_PORT}..."
    (cd "${UI_DIR}" && nohup "${FLUTTER_BIN}" run \
        -d chrome \
        --web-port "${UI_PORT}" \
        --web-hostname 127.0.0.1 \
        > "${FLUTTER_LOG}" 2>&1) &

    # Wait for Flutter to bind the port (up to 30 s)
    READY=false
    for i in $(seq 1 60); do
        if lsof -ti :"${UI_PORT}" &>/dev/null; then
            READY=true
            break
        fi
        sleep 0.5
    done

    if [[ "${READY}" != "true" ]]; then
        die "Flutter dev server did not start within 30 seconds. Check ${FLUTTER_LOG}"
    fi
    success "Flutter dev server is up"

    echo ""
    success "Dev environment ready!"
    echo -e "  ${GREEN}App:${RESET}  http://127.0.0.1:${UI_PORT}"
    echo -e "  ${GREEN}Node:${RESET} http://127.0.0.1:${NODE_PORT}/rpc"
    echo -e "  ${GREEN}Logs:${RESET} ${NODE_LOG}"
    echo -e "        ${FLUTTER_LOG}"
else
    echo ""
    success "Node ready (UI skipped)."
    echo -e "  ${GREEN}Node:${RESET} http://127.0.0.1:${NODE_PORT}/rpc"
    echo -e "  ${GREEN}Log:${RESET}  ${NODE_LOG}"
fi
