#!/usr/bin/env bash
# nyxforge-clean.sh — Remove NyxForge build artifacts and/or node data.
#
# Usage:
#   ./scripts/nyxforge-clean.sh [--include code,data] [--dry-run]
#
# Flags:
#   --include <list>   Comma-separated list of what to remove (default: code).
#                        code  — Cargo target/ and Flutter build/ artifacts
#                        data  — node_data/ directory (wallet keys, chain state)
#                               and node.log
#   --dry-run          Print what would be removed without deleting anything.
#   --help             Show this message and exit.
#
# Examples:
#   ./scripts/nyxforge-clean.sh                       # remove build artifacts
#   ./scripts/nyxforge-clean.sh --include data        # remove node data only
#   ./scripts/nyxforge-clean.sh --include code,data   # full wipe
#   ./scripts/nyxforge-clean.sh --include code,data --dry-run

set -euo pipefail

# ---------------------------------------------------------------------------
# Colours
# ---------------------------------------------------------------------------
RED='\033[0;31m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'
BOLD='\033[1m'; DIM='\033[2m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[clean]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[clean] WARNING:${RESET} $*"; }
removed() { echo -e "  ${RED}✖${RESET} $*"; }
skipped() { echo -e "  ${DIM}– $* (skipped — not found)${RESET}"; }
dry()     { echo -e "  ${YELLOW}~ $* (dry-run — would remove)${RESET}"; }

# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------
INCLUDE_CODE=true
INCLUDE_DATA=false
DRY_RUN=false

# ---------------------------------------------------------------------------
# Locate repo root (script may be called from any directory)
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
usage() {
    sed -n '/^# Usage:/,/^[^#]/p' "${BASH_SOURCE[0]}" | grep '^#' | sed 's/^# \?//'
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            usage
            ;;
        --dry-run)
            DRY_RUN=true
            ;;
        --include)
            [[ $# -ge 2 ]] || { echo "Error: --include requires a value" >&2; exit 1; }
            INCLUDE_CODE=false
            INCLUDE_DATA=false
            IFS=',' read -ra PARTS <<< "$2"
            for part in "${PARTS[@]}"; do
                case "${part// /}" in
                    code) INCLUDE_CODE=true ;;
                    data) INCLUDE_DATA=true ;;
                    *) echo "Error: unknown --include value '${part}' (valid: code, data)" >&2; exit 1 ;;
                esac
            done
            shift
            ;;
        *)
            echo "Error: unknown option '$1'" >&2
            echo "Run with --help for usage." >&2
            exit 1
            ;;
    esac
    shift
done

# ---------------------------------------------------------------------------
# Helper: remove a path (file or directory)
# ---------------------------------------------------------------------------
remove() {
    local target="$1"
    local label="${2:-${target}}"

    if [[ ! -e "${target}" && ! -L "${target}" ]]; then
        skipped "${label}"
        return
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        # Show size estimate for directories
        if [[ -d "${target}" ]]; then
            local size
            size="$(du -sh "${target}" 2>/dev/null | cut -f1 || echo "?")"
            dry "${label}  (${size})"
        else
            dry "${label}"
        fi
        return
    fi

    if [[ -d "${target}" ]]; then
        local size
        size="$(du -sh "${target}" 2>/dev/null | cut -f1 || echo "?")"
        rm -rf "${target}"
        removed "${label}  (freed ~${size})"
    else
        rm -f "${target}"
        removed "${label}"
    fi
}

# ---------------------------------------------------------------------------
# Stop running node process (if any)
# ---------------------------------------------------------------------------
stop_node_if_running() {
    local pids
    pids="$(pgrep -x nyxforge-node 2>/dev/null || true)"
    if [[ -n "${pids}" ]]; then
        if [[ "${DRY_RUN}" == true ]]; then
            warn "nyxforge-node is running (PIDs: ${pids}) — dry-run, not stopping"
            return
        fi
        warn "Stopping running nyxforge-node (PIDs: ${pids})..."
        kill "${pids}" 2>/dev/null || true
        # Give it 2 s to shut down gracefully before we delete files under it.
        sleep 2
        if pgrep -x nyxforge-node &>/dev/null; then
            warn "Process did not exit — sending SIGKILL"
            pkill -9 -x nyxforge-node 2>/dev/null || true
        fi
        info "Node stopped."
    fi
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
echo ""
echo -e "${BOLD}NyxForge clean${RESET}  root: ${REPO_ROOT}"
[[ "${DRY_RUN}" == true ]] && echo -e "${YELLOW}  DRY RUN — nothing will be deleted${RESET}"
echo ""

# Always stop the node before touching data or binaries.
if [[ "${INCLUDE_CODE}" == true || "${INCLUDE_DATA}" == true ]]; then
    stop_node_if_running
fi

# ---------------------------------------------------------------------------
# Code artifacts
# ---------------------------------------------------------------------------
if [[ "${INCLUDE_CODE}" == true ]]; then
    info "Build artifacts:"
    # Cargo/Rust
    remove "${REPO_ROOT}/target"                    "target/           (Cargo build cache)"
    # Flutter
    remove "${REPO_ROOT}/ui/build"                  "ui/build/         (Flutter build output)"
    remove "${REPO_ROOT}/ui/.dart_tool"             "ui/.dart_tool/    (Dart tool cache)"
    remove "${REPO_ROOT}/ui/.flutter-plugins"       "ui/.flutter-plugins"
    remove "${REPO_ROOT}/ui/.flutter-plugins-dependencies" \
                                                    "ui/.flutter-plugins-dependencies"
fi

# ---------------------------------------------------------------------------
# Node data
# ---------------------------------------------------------------------------
if [[ "${INCLUDE_DATA}" == true ]]; then
    echo ""
    info "Node data:"

    # wallet.json contains private keys — extra warning
    WALLET="${REPO_ROOT}/node_data/wallet.json"
    if [[ -f "${WALLET}" ]]; then
        if [[ "${DRY_RUN}" == true ]]; then
            warn "wallet.json contains private keys — dry-run, not removing"
            dry "${WALLET}"
        else
            warn "This will permanently delete your wallet private keys."
            warn "Make sure you have a backup before continuing."
            echo -n "  Type 'yes' to confirm: "
            read -r CONFIRM
            if [[ "${CONFIRM}" != "yes" ]]; then
                info "Aborted — wallet not deleted."
                exit 0
            fi
        fi
    fi

    remove "${REPO_ROOT}/node_data"    "node_data/   (chain state, wallet keys)"
    remove "${REPO_ROOT}/node.log"     "node.log"
fi

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
echo ""
if [[ "${DRY_RUN}" == true ]]; then
    info "Dry run complete — nothing was deleted."
else
    info "Done."
fi
echo ""
