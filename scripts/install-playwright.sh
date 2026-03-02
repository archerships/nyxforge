#!/usr/bin/env bash
# install-playwright.sh -- Install Node.js + Playwright for NyxForge E2E tests
#
# Installs:
#   - nvm (Node version manager)
#   - Node.js LTS
#   - npm dependencies in ui/
#   - Playwright browser binaries (Chromium, Firefox, WebKit)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UI_DIR="$(cd "${SCRIPT_DIR}/../ui" && pwd)"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[playwright]${RESET} $*"; }
success() { echo -e "${GREEN}[playwright]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[playwright]${RESET} $*"; }
die()     { echo -e "${RED}[playwright] ERROR:${RESET} $*" >&2; exit 1; }

command_exists() { command -v "$1" &>/dev/null; }

add_to_shell() {
    local line="${1}"
    local marker="${2}"
    for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
        [[ -f "${rc}" ]] || continue
        if ! grep -qF "${marker}" "${rc}"; then
            echo "" >> "${rc}"
            echo "# ${marker}" >> "${rc}"
            echo "${line}" >> "${rc}"
            info "Added to ${rc}"
        fi
    done
}

# -- 1. Node.js via nvm -------------------------------------------------------
NVM_DIR="${NVM_DIR:-$HOME/.nvm}"

if [[ -d "${NVM_DIR}" ]]; then
    success "nvm already installed"
    # shellcheck source=/dev/null
    source "${NVM_DIR}/nvm.sh"
else
    info "Installing nvm (Node version manager)..."
    curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
    # shellcheck source=/dev/null
    source "${NVM_DIR}/nvm.sh"
    success "nvm installed"
fi

# Load nvm for this session
[[ -s "${NVM_DIR}/nvm.sh" ]] && source "${NVM_DIR}/nvm.sh"

add_to_shell "export NVM_DIR=\"${NVM_DIR}\"" "nyxforge: nvm"
add_to_shell '[[ -s "$NVM_DIR/nvm.sh" ]] && source "$NVM_DIR/nvm.sh"' "nyxforge: nvm-load"

# Install/use LTS Node
if command_exists node; then
    success "Node already installed ($(node --version))"
else
    info "Installing Node.js LTS..."
    nvm install --lts
    nvm use --lts
    success "Node.js installed ($(node --version))"
fi

# -- 2. npm dependencies in ui/ -----------------------------------------------
info "Installing npm dependencies in ui/..."
cd "${UI_DIR}"

if [[ ! -f "package.json" ]]; then
    die "ui/package.json not found -- was it created?"
fi

npm install
success "npm dependencies installed"

# -- 3. Playwright browser binaries -------------------------------------------
info "Installing Playwright browser binaries (Chromium, Firefox, WebKit)..."
npx playwright install --with-deps chromium firefox webkit
success "Playwright browsers installed"

# -- 4. verify -----------------------------------------------------------------
echo ""
success "Playwright setup complete!"
printf "  %-20s %s\n" "node:" "$(node --version)"
printf "  %-20s %s\n" "npm:"  "$(npm --version)"
printf "  %-20s %s\n" "playwright:" "$(npx playwright --version 2>/dev/null || echo 'check PATH')"
echo ""
echo -e "${BOLD}  Run tests:${RESET}"
echo -e "    cd ${UI_DIR}"
echo -e "    npx playwright test                 # all tests"
echo -e "    npx playwright test --ui            # interactive mode"
echo -e "    npx playwright show-report          # open last report"
