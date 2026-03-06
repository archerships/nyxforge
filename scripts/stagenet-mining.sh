#!/usr/bin/env bash
# stagenet-mining.sh -- Start / stop / status of Monero stagenet mining stack
#
# Usage:
#   ./scripts/stagenet-mining.sh start  [--wallet <stagenet-xmr-address>]
#   ./scripts/stagenet-mining.sh stop
#   ./scripts/stagenet-mining.sh status
#   ./scripts/stagenet-mining.sh logs   [monerod|p2pool]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_FILE="$HOME/.config/nyxforge/stagenet/config.env"
LOG_DIR="$HOME/.config/nyxforge/stagenet/logs"
PID_DIR="$HOME/.config/nyxforge/stagenet"

MONEROD_PID="$PID_DIR/monerod.pid"
P2POOL_PID="$PID_DIR/p2pool.pid"
MONEROD_LOG="$LOG_DIR/monerod.log"
P2POOL_LOG="$LOG_DIR/p2pool.log"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[stagenet]${RESET} $*"; }
success() { echo -e "${GREEN}[stagenet]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[stagenet]${RESET} $*"; }
die()     { echo -e "${RED}[stagenet] ERROR:${RESET} $*" >&2; exit 1; }

mkdir -p "$LOG_DIR"

# Load config
[[ -f "$CONFIG_FILE" ]] && source "$CONFIG_FILE"
MONEROD_RPC_PORT="${MONEROD_RPC_PORT:-38081}"
MONEROD_ZMQ_PORT="${MONEROD_ZMQ_PORT:-38083}"
MONEROD_P2P_PORT="${MONEROD_P2P_PORT:-38080}"
P2POOL_STRATUM_PORT="${P2POOL_STRATUM_PORT:-3333}"
P2POOL_P2P_PORT="${P2POOL_P2P_PORT:-37889}"
MONEROD_EXTRA_FLAGS="${MONEROD_EXTRA_FLAGS:-}"
P2POOL_EXTRA_FLAGS="${P2POOL_EXTRA_FLAGS:-}"

MONEROD_BIN=$(command -v monerod 2>/dev/null || echo "")
P2POOL_BIN=$(command -v p2pool 2>/dev/null || echo "$HOME/.local/bin/p2pool")

pid_alive() { [[ -n "$1" ]] && kill -0 "$1" 2>/dev/null; }
read_pid()  { [[ -f "$1" ]] && cat "$1" || echo ""; }

# ---------------------------------------------------------------------------
do_start() {
    local wallet="${1:-}"

    # Resolve wallet address: flag > config > NyxForge node RPC > die
    if [[ -z "$wallet" ]]; then
        wallet="${MINING_WALLET:-}"
    fi
    if [[ -z "$wallet" ]]; then
        wallet=$(curl -sf --max-time 2 -X POST http://127.0.0.1:8888/rpc \
            -H 'Content-Type: application/json' \
            -d '{"method":"wallet.addresses","params":{}}' \
            | python3 -c "import sys,json; print(json.load(sys.stdin).get('result',{}).get('xmr',''))" \
            2>/dev/null || echo "")
    fi
    [[ -n "$wallet" ]] || die "No wallet address. Provide --wallet <addr> or run: ./nyxforge (creates wallet automatically)"

    info "Wallet: ${wallet:0:10}...${wallet: -8}"

    # -- monerod ---------------------------------------------------------------
    local node_pid
    node_pid=$(read_pid "$MONEROD_PID")
    if pid_alive "$node_pid"; then
        success "monerod already running (PID $node_pid)"
    else
        [[ -x "$MONEROD_BIN" ]] || die "monerod not found — run: ./scripts/install-stagenet-mining.sh"
        info "Starting monerod (stagenet)..."
        "$MONEROD_BIN" \
            --stagenet \
            --detach \
            --rpc-bind-port "$MONEROD_RPC_PORT" \
            --p2p-bind-port "$MONEROD_P2P_PORT" \
            --zmq-pub "tcp://127.0.0.1:${MONEROD_ZMQ_PORT}" \
            --log-file "$MONEROD_LOG" \
            --non-interactive \
            $MONEROD_EXTRA_FLAGS
        # monerod --detach writes its own PID; find it via pgrep.
        sleep 2
        local mpid
        mpid=$(pgrep -x monerod 2>/dev/null | head -1 || echo "")
        echo "$mpid" > "$MONEROD_PID"
        success "monerod started (PID $mpid)  logs -> $MONEROD_LOG"
        info "Stagenet blockchain sync will run in the background."
        info "P2Pool can connect before sync completes."
    fi

    # -- p2pool ----------------------------------------------------------------
    local pp_pid
    pp_pid=$(read_pid "$P2POOL_PID")
    if pid_alive "$pp_pid"; then
        success "p2pool already running (PID $pp_pid)"
    else
        [[ -x "$P2POOL_BIN" ]] || die "p2pool not found — run: ./scripts/install-stagenet-mining.sh"
        info "Starting P2Pool (stagenet)..."
        "$P2POOL_BIN" \
            --stagenet \
            --host 127.0.0.1 \
            --rpc-port "$MONEROD_RPC_PORT" \
            --zmq-port "$MONEROD_ZMQ_PORT" \
            --wallet "$wallet" \
            --stratum "0.0.0.0:${P2POOL_STRATUM_PORT}" \
            --p2p "0.0.0.0:${P2POOL_P2P_PORT}" \
            --loglevel 1 \
            $P2POOL_EXTRA_FLAGS \
            >> "$P2POOL_LOG" 2>&1 &
        echo $! > "$P2POOL_PID"
        sleep 1
        success "P2Pool started (PID $!)  logs -> $P2POOL_LOG"
        success "Stratum listening on 127.0.0.1:${P2POOL_STRATUM_PORT}"
    fi

    echo ""
    success "Mining stack ready."
    echo -e "  ${BOLD}Start NyxForge miner:${RESET}  ./nyxforge --mine --threads \$(nproc)"
}

# ---------------------------------------------------------------------------
do_stop() {
    local pp_pid mnode_pid
    pp_pid=$(read_pid "$P2POOL_PID")
    mnode_pid=$(read_pid "$MONEROD_PID")

    if pid_alive "$pp_pid"; then
        info "Stopping P2Pool (PID $pp_pid)..."
        kill "$pp_pid" 2>/dev/null || true
        sleep 1
        kill -0 "$pp_pid" 2>/dev/null && kill -9 "$pp_pid" 2>/dev/null || true
        rm -f "$P2POOL_PID"
        success "P2Pool stopped"
    else
        info "P2Pool: not running"
        rm -f "$P2POOL_PID"
    fi

    if pid_alive "$mnode_pid"; then
        info "Stopping monerod (PID $mnode_pid)..."
        kill "$mnode_pid" 2>/dev/null || true
        # monerod needs a moment to flush its DB gracefully
        for _ in $(seq 1 20); do
            kill -0 "$mnode_pid" 2>/dev/null || break
            sleep 0.5
        done
        kill -0 "$mnode_pid" 2>/dev/null && kill -9 "$mnode_pid" 2>/dev/null || true
        rm -f "$MONEROD_PID"
        success "monerod stopped"
    else
        info "monerod: not running"
        rm -f "$MONEROD_PID"
    fi
}

# ---------------------------------------------------------------------------
do_status() {
    local RULE
    RULE=$(printf '%.0s-' {1..56})
    echo ""
    echo -e "${BOLD}Stagenet mining stack${RESET}  $(date '+%Y-%m-%d %H:%M:%S')"
    echo "$RULE"

    ok()   { echo -e "${GREEN}running${RESET}"; }
    dead() { echo -e "${RED}stopped${RESET}"; }

    uptime_for() { ps -o etime= -p "$1" 2>/dev/null | tr -d ' ' || echo "?"; }

    # monerod
    local node_pid node_status node_detail=""
    node_pid=$(read_pid "$MONEROD_PID")
    if pid_alive "$node_pid"; then
        node_status=$(ok)
        # Try to get sync height via RPC
        local height
        height=$(curl -sf --max-time 2 -X POST "http://127.0.0.1:${MONEROD_RPC_PORT}/json_rpc" \
            -H 'Content-Type: application/json' \
            -d '{"jsonrpc":"2.0","id":"0","method":"get_info"}' \
            | python3 -c "import sys,json; d=json.load(sys.stdin)['result']; print(f\"height {d['height']}/{d['target_height']} (sync {100*d['height']//max(d['target_height'],1)}%)\")" \
            2>/dev/null || echo "syncing...")
        node_detail="PID $node_pid  up $(uptime_for "$node_pid")  $height"
    else
        node_status=$(dead)
    fi
    printf "  ${BOLD}%-16s${RESET}  %s  %s\n" "monerod" "$node_status" "$node_detail"

    # p2pool
    local pp_pid pp_status pp_detail=""
    pp_pid=$(read_pid "$P2POOL_PID")
    if pid_alive "$pp_pid"; then
        pp_status=$(ok)
        pp_detail="PID $pp_pid  up $(uptime_for "$pp_pid")  stratum :${P2POOL_STRATUM_PORT}"
    else
        pp_status=$(dead)
    fi
    printf "  ${BOLD}%-16s${RESET}  %s  %s\n" "p2pool" "$pp_status" "$pp_detail"

    echo "$RULE"
    echo ""
}

# ---------------------------------------------------------------------------
do_logs() {
    local target="${1:-}"
    case "$target" in
        monerod) tail -f "$MONEROD_LOG" ;;
        p2pool)  tail -f "$P2POOL_LOG" ;;
        "")
            echo "==> monerod <=="
            tail -20 "$MONEROD_LOG" 2>/dev/null || echo "(no log yet)"
            echo ""
            echo "==> p2pool <=="
            tail -20 "$P2POOL_LOG" 2>/dev/null || echo "(no log yet)"
            ;;
        *) die "Unknown log target '$target' — use 'monerod' or 'p2pool'" ;;
    esac
}

# ---------------------------------------------------------------------------
# Dispatch
# ---------------------------------------------------------------------------
CMD="${1:-}"
shift || true

WALLET_ARG=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --wallet) WALLET_ARG="${2:-}"; shift ;;
        *) die "Unknown option: $1" ;;
    esac
    shift
done

case "$CMD" in
    start)  do_start "$WALLET_ARG" ;;
    stop)   do_stop ;;
    status) do_status ;;
    logs)   do_logs "${WALLET_ARG}" ;;
    "")     echo "Usage: $0 {start|stop|status|logs} [--wallet <addr>]"; exit 1 ;;
    *)      die "Unknown command: $CMD  (start|stop|status|logs)" ;;
esac
