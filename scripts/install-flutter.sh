#!/usr/bin/env bash
# install-flutter.sh -- Install Flutter SDK (stable, macOS ARM64/x64)
#
# Idempotent: safe to re-run; skips steps already done.
# Installs to: $FLUTTER_HOME  (default: ~/development/flutter)
# Adds to PATH via: ~/.zshrc and ~/.bashrc

set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[flutter]${RESET} $*"; }
success() { echo -e "${GREEN}[flutter]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[flutter]${RESET} $*"; }
die()     { echo -e "${RED}[flutter] ERROR:${RESET} $*" >&2; exit 1; }

# -- config -------------------------------------------------------------------
FLUTTER_HOME="${FLUTTER_HOME:-$HOME/development/flutter}"
RELEASES_URL="https://storage.googleapis.com/flutter_infra_release/releases/releases_macos.json"

# -- detect arch --------------------------------------------------------------
ARCH="$(uname -m)"
if [[ "${ARCH}" == "arm64" ]]; then
    ARCH_SUFFIX="arm64_"
else
    ARCH_SUFFIX=""          # x64 has no suffix in the filename
fi

# -- helpers ------------------------------------------------------------------
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

# -- 1. check existing install ------------------------------------------------
if [[ -x "${FLUTTER_HOME}/bin/flutter" ]]; then
    EXISTING_VER="$("${FLUTTER_HOME}/bin/flutter" --version --machine 2>/dev/null \
        | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d["frameworkVersion"])' 2>/dev/null \
        || echo "unknown")"
    success "Flutter already installed at ${FLUTTER_HOME} (version: ${EXISTING_VER})"
    success "Delete ${FLUTTER_HOME} and re-run to reinstall."
    SKIP_DOWNLOAD=true
else
    SKIP_DOWNLOAD=false
fi

# -- 2. download --------------------------------------------------------------
if [[ "${SKIP_DOWNLOAD}" == "false" ]]; then
    info "Fetching latest stable Flutter release metadata..."

    RELEASES_JSON="$(curl -fsSL "${RELEASES_URL}")"
    CURRENT_HASH="$(echo "${RELEASES_JSON}" | python3 -c \
        'import sys,json; d=json.load(sys.stdin); print(d["current_release"]["stable"])')"

    # Find the download URL for our arch
    DOWNLOAD_URL="$(echo "${RELEASES_JSON}" | python3 -c "
import sys, json
d = json.load(sys.stdin)
arch = '${ARCH_SUFFIX}'
for r in d['releases']:
    if r['hash'] == '${CURRENT_HASH}' and arch in r.get('archive', ''):
        print('https://storage.googleapis.com/flutter_infra_release/releases/' + r['archive'])
        break
")"

    if [[ -z "${DOWNLOAD_URL}" ]]; then
        # Fallback: grab the first stable match for our arch
        DOWNLOAD_URL="$(echo "${RELEASES_JSON}" | python3 -c "
import sys, json
d = json.load(sys.stdin)
arch = '${ARCH_SUFFIX}'
for r in d['releases']:
    if r['channel'] == 'stable' and arch in r.get('archive', ''):
        print('https://storage.googleapis.com/flutter_infra_release/releases/' + r['archive'])
        break
")"
    fi

    [[ -n "${DOWNLOAD_URL}" ]] || die "Could not determine download URL for arch: ${ARCH}"

    VERSION="$(echo "${DOWNLOAD_URL}" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
    info "Downloading Flutter ${VERSION} for ${ARCH}..."
    info "URL: ${DOWNLOAD_URL}"

    TMPDIR_DL="$(mktemp -d)"
    ZIPFILE="${TMPDIR_DL}/flutter.zip"

    curl -fL --progress-bar "${DOWNLOAD_URL}" -o "${ZIPFILE}"

    info "Extracting to ${FLUTTER_HOME}..."
    mkdir -p "$(dirname "${FLUTTER_HOME}")"
    unzip -q "${ZIPFILE}" -d "$(dirname "${FLUTTER_HOME}")"
    rm -rf "${TMPDIR_DL}"

    success "Flutter ${VERSION} extracted to ${FLUTTER_HOME}"
fi

# -- 3. PATH ------------------------------------------------------------------
FLUTTER_BIN="${FLUTTER_HOME}/bin"
if ! echo "${PATH}" | grep -q "${FLUTTER_BIN}"; then
    add_to_shell "export PATH=\"${FLUTTER_BIN}:\$PATH\"" "nyxforge: flutter"
    export PATH="${FLUTTER_BIN}:${PATH}"
    info "PATH updated for this session"
fi

FLUTTER="${FLUTTER_BIN}/flutter"

# -- 4. enable web ------------------------------------------------------------
info "Enabling Flutter web support..."
"${FLUTTER}" config --enable-web --no-analytics 2>/dev/null || true

# -- 5. disable analytics -----------------------------------------------------
"${FLUTTER}" config --no-analytics 2>/dev/null || true
"${FLUTTER}" config --no-cli-animations 2>/dev/null || true

# -- 6. pre-cache web artifacts -----------------------------------------------
info "Pre-caching Flutter web artifacts (this takes a minute)..."
"${FLUTTER}" precache --web 2>/dev/null || true

# -- 7. PATH reminder (dart is bundled with Flutter) --------------------------
add_to_shell "export PATH=\"${FLUTTER_BIN}:\$PATH\"" "nyxforge: dart (bundled with flutter)"

# -- 8. doctor ----------------------------------------------------------------
info "Running flutter doctor..."
"${FLUTTER}" doctor --android-licenses 2>/dev/null || true
"${FLUTTER}" doctor -v 2>&1 | grep -v "^$" | head -60 || true

echo ""
success "Flutter setup complete!"
echo -e "${BOLD}  SDK location:${RESET} ${FLUTTER_HOME}"
echo -e "${BOLD}  Next step:${RESET}   Restart your shell (or: source ~/.zshrc)"
echo -e "${BOLD}  Test with:${RESET}   flutter --version"
