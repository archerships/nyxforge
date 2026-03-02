#!/usr/bin/env bash
# install-rust-wasm.sh -- Install Rust toolchain + WASM targets for NyxForge
#
# Installs:
#   - rustup + cargo (latest stable)
#   - wasm32-unknown-unknown target
#   - wasm-pack   (build Rust -> browser WASM)
#   - trunk       (dev server + bundler for Rust WASM)
#   - cargo-watch (live recompile during dev)
#   - wasm-bindgen-cli
#   - cargo-expand (macro debugging)

set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[rust-wasm]${RESET} $*"; }
success() { echo -e "${GREEN}[rust-wasm]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[rust-wasm]${RESET} $*"; }
die()     { echo -e "${RED}[rust-wasm] ERROR:${RESET} $*" >&2; exit 1; }

command_exists() { command -v "$1" &>/dev/null; }

cargo_install_if_missing() {
    local bin="${1}"; shift
    if command_exists "${bin}"; then
        success "${bin} already installed ($(command -v "${bin}"))"
    else
        info "Installing ${bin}..."
        cargo install "$@"
        success "${bin} installed"
    fi
}

# -- 1. rustup / cargo --------------------------------------------------------
if command_exists rustup; then
    success "rustup already installed"
    info "Updating Rust toolchain..."
    rustup update stable
else
    info "Installing rustup + stable Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --no-modify-path
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
    success "rustup installed"
fi

# Ensure cargo is on PATH for this session
[[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"

command_exists cargo || die "cargo not found after rustup install; check PATH"

RUST_VER="$(rustc --version)"
success "Using ${RUST_VER}"

# -- 2. WASM target -----------------------------------------------------------
info "Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown
success "wasm32-unknown-unknown target ready"

# -- 3. wasm-pack -------------------------------------------------------------
if command_exists wasm-pack; then
    success "wasm-pack already installed ($(wasm-pack --version))"
else
    info "Installing wasm-pack via official installer..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    success "wasm-pack installed"
fi

# -- 4. trunk -----------------------------------------------------------------
cargo_install_if_missing trunk trunk

# -- 5. cargo-watch -----------------------------------------------------------
cargo_install_if_missing cargo-watch cargo-watch

# -- 6. wasm-bindgen-cli ------------------------------------------------------
if command_exists wasm-bindgen; then
    success "wasm-bindgen already installed ($(wasm-bindgen --version))"
else
    info "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
    success "wasm-bindgen-cli installed"
fi

# -- 7. cargo-expand ----------------------------------------------------------
cargo_install_if_missing cargo-expand cargo-expand

# -- 8. PATH reminder ---------------------------------------------------------
CARGO_BIN="$HOME/.cargo/bin"
for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
    [[ -f "${rc}" ]] || continue
    if ! grep -qF 'cargo/bin' "${rc}"; then
        echo "" >> "${rc}"
        echo "# nyxforge: rust/cargo" >> "${rc}"
        echo "export PATH=\"${CARGO_BIN}:\$PATH\"" >> "${rc}"
        info "Added \$HOME/.cargo/bin to ${rc}"
    fi
done

echo ""
success "Rust + WASM toolchain ready!"
echo -e "${BOLD}  rustc:${RESET}       $(rustc --version)"
echo -e "${BOLD}  cargo:${RESET}       $(cargo --version)"
echo -e "${BOLD}  wasm-pack:${RESET}   $(wasm-pack --version 2>/dev/null || echo 'check PATH')"
echo -e "${BOLD}  trunk:${RESET}       $(trunk --version 2>/dev/null || echo 'check PATH')"
echo ""
echo -e "${BOLD}  Restart shell or run:${RESET}  source ~/.cargo/env"
