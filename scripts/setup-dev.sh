#!/usr/bin/env bash
# setup-dev.sh -- Bootstrap the full NyxForge development environment
#
# Usage:
#   ./scripts/setup-dev.sh           # install everything
#   ./scripts/setup-dev.sh --rust    # Rust + WASM only
#   ./scripts/setup-dev.sh --flutter # Flutter only
#   ./scripts/setup-dev.sh --check   # verify installs without changing anything
#
# Re-running is safe: all sub-scripts are idempotent.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[setup]${RESET} $*"; }
success() { echo -e "${GREEN}[setup]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[setup]${RESET} $*"; }
die()     { echo -e "${RED}[setup] ERROR:${RESET} $*" >&2; exit 1; }
header()  { echo -e "\n${BOLD}=== $* ===${RESET}"; }

command_exists() { command -v "$1" &>/dev/null; }

# -- parse args ---------------------------------------------------------------
DO_RUST=true
DO_FLUTTER=true
CHECK_ONLY=false

for arg in "$@"; do
    case "${arg}" in
        --rust)    DO_FLUTTER=false ;;
        --flutter) DO_RUST=false ;;
        --check)   CHECK_ONLY=true; DO_RUST=false; DO_FLUTTER=false ;;
        --help|-h)
            echo "Usage: $0 [--rust] [--flutter] [--check]"
            exit 0 ;;
        *) die "Unknown argument: ${arg}" ;;
    esac
done

# -- system checks ------------------------------------------------------------
header "System"
info "OS:    $(sw_vers -productName) $(sw_vers -productVersion)"
info "Arch:  $(uname -m)"
info "Shell: ${SHELL}"

# Require Xcode CLI tools (needed for git, clang, etc.)
if ! xcode-select -p &>/dev/null; then
    warn "Xcode Command Line Tools not found -- installing..."
    xcode-select --install || true
    die "Re-run this script after Xcode CLI tools finish installing"
fi
success "Xcode CLI tools: $(xcode-select -p)"

# Homebrew check (informational only)
if command_exists brew; then
    success "Homebrew: $(brew --version | head -1)"
else
    warn "Homebrew not found; some optional tools may not install"
fi

# -- rust + wasm --------------------------------------------------------------
if [[ "${DO_RUST}" == "true" ]]; then
    header "Rust + WASM Toolchain"
    bash "${SCRIPT_DIR}/install-rust-wasm.sh"
fi

# -- flutter ------------------------------------------------------------------
if [[ "${DO_FLUTTER}" == "true" ]]; then
    header "Flutter SDK"
    bash "${SCRIPT_DIR}/install-flutter.sh"
fi

# -- playwright ---------------------------------------------------------------
if [[ "${DO_FLUTTER}" == "true" ]]; then
    header "Playwright (E2E tests)"
    bash "${SCRIPT_DIR}/install-playwright.sh"
fi

# -- git hooks ----------------------------------------------------------------
header "Git Hooks"
HOOKS_SRC="${SCRIPT_DIR}/hooks"
HOOKS_DST="${PROJECT_ROOT}/.git/hooks"
if [[ -d "${HOOKS_DST}" ]]; then
    for hook in "${HOOKS_SRC}"/*; do
        name="$(basename "${hook}")"
        ln -sf "../../scripts/hooks/${name}" "${HOOKS_DST}/${name}"
        success "Installed hook: ${name}"
    done
else
    warn ".git/hooks not found — skipping hook installation"
fi

# -- check / summary ----------------------------------------------------------
header "Environment Check"

check() {
    local label="${1}"; local cmd="${2}"
    if ver="$(eval "${cmd}" 2>/dev/null)"; then
        printf "  ${GREEN}[OK]${RESET}  %-20s %s\n" "${label}" "${ver}"
    else
        printf "  ${RED}[!!]${RESET}  %-20s ${RED}not found${RESET}\n" "${label}"
    fi
}

check "rustc"     "rustc --version"
check "cargo"     "cargo --version"
check "wasm-pack" "wasm-pack --version"
check "trunk"     "trunk --version"
check "flutter"   "flutter --version --machine 2>/dev/null | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d[\"frameworkVersion\"])'"
check "dart"       "dart --version 2>&1 | head -1"
check "node"       "node --version"
check "playwright" "npx --yes playwright --version 2>/dev/null | head -1"
check "git"        "git --version"
check "python3"   "python3 --version"
check "curl"      "curl --version | head -1"

echo ""

# Warn if Chrome is missing (needed for flutter web dev)
if ! command_exists google-chrome && ! command_exists chromium && \
   ! ls "/Applications/Google Chrome.app" &>/dev/null; then
    warn "Google Chrome not found -- required for 'flutter run -d chrome'"
    warn "Install from: https://www.google.com/chrome/"
fi

echo ""
success "Dev environment setup complete for NyxForge!"
echo -e "${BOLD}  Project root:${RESET} ${PROJECT_ROOT}"
echo ""
echo -e "  ${BOLD}Next steps:${RESET}"
echo -e "    1.  source ~/.zshrc                      # reload PATH"
echo -e "    2.  cd ${PROJECT_ROOT}"
echo -e "    3.  cargo build --workspace               # build Rust crates"
echo -e "    4.  cd ui && flutter run -d chrome        # run Flutter UI"
