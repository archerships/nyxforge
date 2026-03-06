#!/usr/bin/env bash
# install-stagenet-mining.sh -- Install and configure Monero stagenet mining stack
#
# Installs:
#   monerod      Monero daemon (stagenet) — syncs the stagenet blockchain
#   P2Pool       Stratum server that bridges monerod to NyxForge's miner
#
# After running this script:
#   ./scripts/stagenet-mining.sh start          # start monerod + p2pool
#   ./nyxforge --mine --threads <n>             # start NyxForge with mining
#
# Flags:
#   --autostart     Install launchd agents so monerod + p2pool start at login
#   --check         Verify existing installs without changing anything
#   --help
#
# Re-running is safe: already-installed components are skipped.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CONFIG_DIR="$HOME/.config/nyxforge/stagenet"
INSTALL_DIR="$HOME/.local/bin"
LAUNCH_AGENTS="$HOME/Library/LaunchAgents"
MONEROD_DATA="$HOME/.bitmonero/stagenet"
P2POOL_DATA="$HOME/.p2pool"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[stagenet]${RESET} $*"; }
success() { echo -e "${GREEN}[stagenet]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[stagenet]${RESET} $*"; }
die()     { echo -e "${RED}[stagenet] ERROR:${RESET} $*" >&2; exit 1; }
header()  { echo -e "\n${BOLD}=== $* ===${RESET}"; }

command_exists() { command -v "$1" &>/dev/null; }

# ---------------------------------------------------------------------------
# Parse args
# ---------------------------------------------------------------------------
AUTOSTART=false
CHECK_ONLY=false

for arg in "$@"; do
    case "$arg" in
        --autostart) AUTOSTART=true ;;
        --check)     CHECK_ONLY=true ;;
        --help|-h)
            sed -n '/^# /{ s/^# \{0,1\}//; p }' "$0" | head -20
            exit 0 ;;
        *) die "Unknown argument: ${arg}  (use --help)" ;;
    esac
done

# ---------------------------------------------------------------------------
# Detect architecture
# ---------------------------------------------------------------------------
ARCH=$(uname -m)
case "$ARCH" in
    arm64|aarch64) P2POOL_ARCH="aarch64" ;;
    x86_64)        P2POOL_ARCH="x86_64" ;;
    *) die "Unsupported architecture: $ARCH" ;;
esac

# ---------------------------------------------------------------------------
# Helper: check mode summary
# ---------------------------------------------------------------------------
if [[ "$CHECK_ONLY" == true ]]; then
    header "Stagenet Mining — Installation Check"
    check_item() {
        local label="$1" cmd="$2"
        if ver=$(eval "$cmd" 2>/dev/null); then
            printf "  ${GREEN}[OK]${RESET}  %-20s %s\n" "$label" "$ver"
        else
            printf "  ${RED}[--]${RESET}  %-20s not found\n" "$label"
        fi
    }
    check_item "monerod"  "monerod --version 2>&1 | head -1"
    check_item "p2pool"   "p2pool --version 2>&1 | head -1 || $INSTALL_DIR/p2pool --version 2>&1 | head -1"
    check_item "stagenet-mining.sh" "ls $SCRIPT_DIR/stagenet-mining.sh && echo present"
    echo ""
    exit 0
fi

# ---------------------------------------------------------------------------
# 1. Homebrew check
# ---------------------------------------------------------------------------
header "Prerequisites"
command_exists brew || die "Homebrew is required. Install from https://brew.sh"
success "Homebrew: $(brew --version | head -1)"

mkdir -p "$INSTALL_DIR" "$CONFIG_DIR" "$P2POOL_DATA"

# ---------------------------------------------------------------------------
# 2. monerod
# ---------------------------------------------------------------------------
header "monerod (Monero daemon)"

if command_exists monerod; then
    MONEROD_VER=$(monerod --version 2>&1 | head -1)
    success "monerod already installed: $MONEROD_VER"
else
    info "Installing monero via Homebrew..."
    brew install monero
    MONEROD_VER=$(monerod --version 2>&1 | head -1)
    success "monerod installed: $MONEROD_VER"
fi

MONEROD_BIN=$(command -v monerod)

# ---------------------------------------------------------------------------
# 3. P2Pool
# ---------------------------------------------------------------------------
header "P2Pool (stratum bridge)"

P2POOL_BIN=""
if command_exists p2pool; then
    P2POOL_BIN=$(command -v p2pool)
    success "p2pool already on PATH: $P2POOL_BIN"
elif [[ -x "$INSTALL_DIR/p2pool" ]]; then
    P2POOL_BIN="$INSTALL_DIR/p2pool"
    success "p2pool already installed: $P2POOL_BIN"
else
    info "Fetching latest P2Pool release info from GitHub..."
    RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/SChernykh/p2pool/releases/latest")
    P2POOL_VERSION=$(echo "$RELEASE_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin)['tag_name'])")
    info "Latest P2Pool: $P2POOL_VERSION"

    # Find the macOS asset matching our architecture.
    ASSET_URL=$(echo "$RELEASE_JSON" | python3 -c "
import sys, json, re
assets = json.load(sys.stdin)['assets']
arch = '${P2POOL_ARCH}'
for a in assets:
    n = a['name'].lower()
    if 'macos' in n and arch in n and n.endswith('.tar.gz'):
        print(a['browser_download_url'])
        break
")
    [[ -n "$ASSET_URL" ]] || die "Could not find a macOS ${P2POOL_ARCH} P2Pool asset in the release"
    info "Downloading: $ASSET_URL"

    TMP_DIR=$(mktemp -d)
    TARBALL="$TMP_DIR/p2pool.tar.gz"
    curl -fsSL -o "$TARBALL" "$ASSET_URL"

    tar -xzf "$TARBALL" -C "$TMP_DIR"
    P2POOL_EXTRACTED=$(find "$TMP_DIR" -name "p2pool" -type f | head -1)
    [[ -n "$P2POOL_EXTRACTED" ]] || die "p2pool binary not found in archive"

    cp "$P2POOL_EXTRACTED" "$INSTALL_DIR/p2pool"
    chmod +x "$INSTALL_DIR/p2pool"
    rm -rf "$TMP_DIR"

    P2POOL_BIN="$INSTALL_DIR/p2pool"
    success "P2Pool $P2POOL_VERSION installed to $P2POOL_BIN"

    # Add ~/.local/bin to PATH if missing.
    for rc in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.zprofile" "$HOME/.bash_profile"; do
        [[ -f "$rc" ]] || continue
        if ! grep -qF '.local/bin' "$rc"; then
            echo "" >> "$rc"
            echo "# nyxforge: local binaries" >> "$rc"
            echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$rc"
            info "Added ~/.local/bin to $rc"
        fi
    done
fi

# ---------------------------------------------------------------------------
# 4. Write config
# ---------------------------------------------------------------------------
header "Configuration"

CONFIG_FILE="$CONFIG_DIR/config.env"
if [[ -f "$CONFIG_FILE" ]]; then
    success "Config already exists: $CONFIG_FILE"
else
    cat > "$CONFIG_FILE" <<ENVEOF
# NyxForge stagenet mining configuration
# Edit this file to change endpoints, wallet address, or thread count.

# Stagenet monerod RPC and ZMQ ports (defaults for --stagenet)
MONEROD_RPC_PORT=38081
MONEROD_ZMQ_PORT=38083
MONEROD_P2P_PORT=38080

# P2Pool stratum port (NyxForge miner connects here)
P2POOL_STRATUM_PORT=3333
P2POOL_P2P_PORT=37889

# Wallet address for mining payouts.
# Leave blank and the nyxforge launch script will fill it in automatically.
MINING_WALLET=""

# Extra flags for monerod (e.g. --prune-blockchain)
MONEROD_EXTRA_FLAGS=""

# Extra flags for p2pool
P2POOL_EXTRA_FLAGS=""
ENVEOF
    success "Config written: $CONFIG_FILE"
fi

info "Data directories:"
info "  monerod stagenet blockchain: $MONEROD_DATA"
info "  p2pool data:                 $P2POOL_DATA"
mkdir -p "$MONEROD_DATA" "$P2POOL_DATA"

# ---------------------------------------------------------------------------
# 5. Write stagenet-mining.sh management script
# ---------------------------------------------------------------------------
header "Management script"

MGMT_SCRIPT="$SCRIPT_DIR/stagenet-mining.sh"

cat > "$MGMT_SCRIPT" <<'SCRIPTEOF'
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
SCRIPTEOF

chmod +x "$MGMT_SCRIPT"
success "Management script: $MGMT_SCRIPT"

# ---------------------------------------------------------------------------
# 6. Optional launchd auto-start agents (macOS)
# ---------------------------------------------------------------------------
if [[ "$AUTOSTART" == true ]]; then
    header "launchd auto-start agents"

    MONEROD_PLIST="$LAUNCH_AGENTS/io.nyxforge.monerod-stagenet.plist"
    P2POOL_PLIST="$LAUNCH_AGENTS/io.nyxforge.p2pool-stagenet.plist"
    mkdir -p "$LAUNCH_AGENTS"

    # monerod plist
    cat > "$MONEROD_PLIST" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>             <string>io.nyxforge.monerod-stagenet</string>
    <key>ProgramArguments</key>
    <array>
        <string>${MONEROD_BIN}</string>
        <string>--stagenet</string>
        <string>--non-interactive</string>
        <string>--rpc-bind-port</string> <string>${MONEROD_RPC_PORT:-38081}</string>
        <string>--p2p-bind-port</string> <string>${MONEROD_P2P_PORT:-38080}</string>
        <string>--zmq-pub</string> <string>tcp://127.0.0.1:${MONEROD_ZMQ_PORT:-38083}</string>
    </array>
    <key>RunAtLoad</key>         <true/>
    <key>KeepAlive</key>         <true/>
    <key>StandardOutPath</key>   <string>${LOG_DIR}/monerod.log</string>
    <key>StandardErrorPath</key> <string>${LOG_DIR}/monerod.log</string>
</dict>
</plist>
PLIST
    launchctl unload "$MONEROD_PLIST" 2>/dev/null || true
    launchctl load -w "$MONEROD_PLIST"
    success "monerod launchd agent installed and started"

    info "P2Pool autostart requires a wallet address."
    info "Run once manually to populate MINING_WALLET in $CONFIG_FILE,"
    info "then re-run with --autostart to install the P2Pool agent."
fi

# ---------------------------------------------------------------------------
# 7. Summary
# ---------------------------------------------------------------------------
header "Done"
echo ""
echo -e "  ${BOLD}monerod:${RESET}   $(monerod --version 2>&1 | head -1)"
echo -e "  ${BOLD}p2pool:${RESET}    $(${P2POOL_BIN} --version 2>&1 | head -1 || echo 'check PATH')"
echo -e "  ${BOLD}config:${RESET}    $CONFIG_FILE"
echo ""
echo -e "  ${BOLD}Next steps:${RESET}"
echo -e "    1.  ./scripts/stagenet-mining.sh start"
echo -e "        (add --wallet <stagenet-addr> the first time, or let NyxForge fill it in)"
echo -e "    2.  ./nyxforge --mine --threads \$(sysctl -n hw.logicalcpu)"
echo ""
echo -e "  ${BOLD}Useful commands:${RESET}"
echo -e "    ./scripts/stagenet-mining.sh status"
echo -e "    ./scripts/stagenet-mining.sh logs monerod"
echo -e "    ./scripts/stagenet-mining.sh logs p2pool"
echo -e "    ./scripts/stagenet-mining.sh stop"
echo ""
warn "First sync: monerod will download the stagenet blockchain (~2-3 GB)."
warn "Mining starts as soon as P2Pool connects to monerod, even before full sync."
echo ""
