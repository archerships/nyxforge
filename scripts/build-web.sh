#!/usr/bin/env bash
# build-web.sh -- Build a fully self-contained, de-googled Flutter WASM bundle
#
# Output: ui/build/web/
#
# De-googling steps applied:
#   1. --wasm: Impeller renderer compiled into the binary (no CanvasKit CDN call)
#   2. Fonts are bundled from local assets (no Google Fonts CDN)
#   3. index.html patched with local canvasKitBaseUrl (guards JS-mode fallback)
#   4. Audit: scan output for external URLs and fail if any are found
#   5. Bundle size report

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
UI_DIR="${PROJECT_ROOT}/ui"
BUILD_DIR="${UI_DIR}/build/web"

export PATH="$HOME/development/flutter/bin:$HOME/.cargo/bin:$PATH"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[build]${RESET} $*"; }
success() { echo -e "${GREEN}[build]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[build]${RESET} $*"; }
die()     { echo -e "${RED}[build] ERROR:${RESET} $*" >&2; exit 1; }

# -- args ---------------------------------------------------------------------
SKIP_AUDIT=false
for arg in "$@"; do
    case "${arg}" in
        --skip-audit) SKIP_AUDIT=true ;;
        --help|-h)
            echo "Usage: $0 [--skip-audit]"
            exit 0 ;;
        *) die "Unknown argument: ${arg}" ;;
    esac
done

cd "${UI_DIR}"

# -- 1. flutter analytics off (build-time only, not shipped to users) ----------
info "Disabling Flutter build-time analytics..."
flutter config --no-analytics 2>/dev/null || true

# -- 2. clean previous build --------------------------------------------------
info "Cleaning previous build..."
flutter clean 2>/dev/null || true

# -- 3. get dependencies ------------------------------------------------------
info "Fetching dependencies..."
flutter pub get

# -- 4. build WASM ------------------------------------------------------------
# --wasm:          Dart2Wasm + Impeller renderer compiled into binary
#                  No CanvasKit loaded from CDN at runtime
# --release:       Full optimisation, tree shaking, no debug info
# --no-source-maps: Don't ship source maps (smaller, no code exposure)
info "Building Flutter WASM bundle (release)..."
flutter build web \
    --wasm \
    --release \
    --no-source-maps \
    --pwa-strategy=none

success "Flutter WASM build complete"

# -- 5. self-host canvaskit (guards against JS-mode fallback) -----------------
# The --wasm build uses Impeller and does NOT load CanvasKit.
# This step handles the non-WASM JS fallback path in case it is ever used.
info "Self-hosting CanvasKit (JS-mode fallback guard)..."

FLUTTER_ROOT="$(flutter --version --machine 2>/dev/null \
    | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d["flutterRoot"])' \
    2>/dev/null || echo "")"

CANVASKIT_SRC=""
if [[ -n "${FLUTTER_ROOT}" ]]; then
    CANVASKIT_SRC="${FLUTTER_ROOT}/bin/cache/flutter_web_sdk/canvaskit"
fi

if [[ -d "${CANVASKIT_SRC}" ]]; then
    CANVASKIT_DST="${BUILD_DIR}/canvaskit"
    mkdir -p "${CANVASKIT_DST}"
    cp -r "${CANVASKIT_SRC}/." "${CANVASKIT_DST}/"
    success "CanvasKit copied to build/web/canvaskit/"

    # Patch index.html to use local canvasKitBaseUrl
    INDEX="${BUILD_DIR}/index.html"
    if [[ -f "${INDEX}" ]] && ! grep -q "canvasKitBaseUrl" "${INDEX}"; then
        # Insert config before the flutter.js loader script tag
        python3 - "${INDEX}" <<'PYEOF'
import sys, re

path = sys.argv[1]
with open(path) as f:
    html = f.read()

patch = '''  <script>
    // De-googled: serve CanvasKit from local path, not unpkg.com CDN
    window.flutterConfiguration = {
      canvasKitBaseUrl: "/canvaskit/"
    };
  </script>
'''

# Insert before the first <script> tag that loads flutter
html = re.sub(r'(<script[^>]*flutter\.js[^>]*>)', patch + r'\1', html, count=1)
with open(path, 'w') as f:
    f.write(html)
print("Patched index.html with local canvasKitBaseUrl")
PYEOF
    fi
else
    warn "CanvasKit source not found at '${CANVASKIT_SRC}' -- skipping copy"
    warn "The --wasm build does not need CanvasKit; this only affects JS-mode fallback"
fi

# -- 6. verify fonts are local ------------------------------------------------
info "Checking font assets..."
FONT_MANIFEST="${BUILD_DIR}/assets/FontManifest.json"
if [[ -f "${FONT_MANIFEST}" ]]; then
    # Fail if any font entry still points to an external URL
    if grep -qi "fonts.gstatic\|fonts.googleapis\|unpkg.com\|cdn\." "${FONT_MANIFEST}"; then
        die "FontManifest.json contains external font URLs -- bundle fonts locally"
    fi
    success "Fonts are self-contained"
else
    warn "FontManifest.json not found -- skipping font check"
fi

# -- 7. external URL audit ----------------------------------------------------
if [[ "${SKIP_AUDIT}" == "false" ]]; then
    info "Auditing build output for external URLs..."

    EXTERNAL_URLS=(
        "fonts.googleapis.com"
        "fonts.gstatic.com"
        "unpkg.com"
        "cdn.jsdelivr.net"
        "storage.googleapis.com"
        "firebase"
        "analytics"
        "gtag"
        "googletagmanager"
        "doubleclick"
    )

    FOUND_EXTERNAL=false
    for url in "${EXTERNAL_URLS[@]}"; do
        # Search JS/HTML/JSON files (skip .wasm binary)
        matches="$(grep -rl "${url}" "${BUILD_DIR}" \
            --include="*.html" \
            --include="*.js" \
            --include="*.json" \
            --include="*.mjs" \
            2>/dev/null || true)"
        if [[ -n "${matches}" ]]; then
            warn "External URL '${url}' found in:"
            echo "${matches}" | while read -r f; do warn "  ${f}"; done
            FOUND_EXTERNAL=true
        fi
    done

    if [[ "${FOUND_EXTERNAL}" == "true" ]]; then
        die "Build contains external URLs -- see warnings above"
    fi

    success "Audit passed: no external URLs found in build output"
fi

# -- 8. bundle size report ----------------------------------------------------
echo ""
echo -e "${BOLD}=== Bundle Size Report ===${RESET}"

total_kb() {
    du -sk "${1}" 2>/dev/null | awk '{print $1}' || echo "?"
}

printf "  %-35s %s\n" "Total build/web/" "$(total_kb "${BUILD_DIR}") KB"

# Find the main WASM file
WASM_FILE="$(find "${BUILD_DIR}" -name "*.wasm" | head -1)"
if [[ -n "${WASM_FILE}" ]]; then
    wasm_kb="$(du -sk "${WASM_FILE}" | awk '{print $1}')"
    printf "  %-35s %s\n" "$(basename "${WASM_FILE}")" "${wasm_kb} KB"
fi

# Find the main JS bootstrap
MJS_FILE="$(find "${BUILD_DIR}" -name "*.mjs" | head -1)"
if [[ -n "${MJS_FILE}" ]]; then
    mjs_kb="$(du -sk "${MJS_FILE}" | awk '{print $1}')"
    printf "  %-35s %s\n" "$(basename "${MJS_FILE}")" "${mjs_kb} KB"
fi

ASSETS_KB="$(total_kb "${BUILD_DIR}/assets")"
printf "  %-35s %s\n" "assets/" "${ASSETS_KB} KB"

echo ""
success "De-googled WASM build ready at: ${BUILD_DIR}"
echo -e "${BOLD}  Serve with:${RESET}"
echo -e "    cd ${BUILD_DIR} && python3 -m http.server 8080"
echo -e "    # or: npx serve -s . -l 8080"
echo ""
echo -e "${BOLD}  COOP/COEP headers required for SharedArrayBuffer (WASM threads):${RESET}"
echo -e "    Cross-Origin-Opener-Policy: same-origin"
echo -e "    Cross-Origin-Embedder-Policy: require-corp"
