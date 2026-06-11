#!/usr/bin/env bash
# install.sh — Build and install the `fl` (ForceLoop) binary locally.
#
# Usage:
#   ./install.sh                # build & install `fl` to ~/.cargo/bin
#   ./install.sh --uninstall    # remove the installed `fl` binary
#   ./install.sh --help         # show this help
#
# Install location: ${CARGO_HOME:-$HOME/.cargo}/bin
# Binary name:      fl   (configured via [[bin]] in Cargo.toml)
# Source:           local path (this directory)
#
# This is a one-off development helper (per CLAUDE.md "一次性脚本" exemption).
# Production installs should go through `cargo install --git ...` or a
# published crate, not this script.

set -euo pipefail

# --- Constants ---------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
readonly BINARY_NAME="fl"
readonly INSTALL_ROOT="${CARGO_HOME:-$HOME/.cargo}/bin"
readonly INSTALL_PATH="$INSTALL_ROOT/$BINARY_NAME"

# --- Helpers -----------------------------------------------------------------
log()  { printf '\033[1;34m[install]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[install]\033[0m %s\n' "$*" >&2; }
err()  { printf '\033[1;31m[install]\033[0m %s\n' "$*" >&2; }

usage() {
  cat <<EOF
Usage: $0 [--uninstall | --help]

Build and install the \`$BINARY_NAME\` binary from local source to
\`$INSTALL_ROOT\`.

Options:
  --uninstall, -u   Remove the installed \`$BINARY_NAME\` binary
  --help, -h        Show this help message

The binary name \`$BINARY_NAME\` is configured in Cargo.toml via
\`[[bin]] name = "$BINARY_NAME"\`; this script reads the same constant.
EOF
}

# --- Argument parsing --------------------------------------------------------
ACTION="install"
for arg in "$@"; do
  case "$arg" in
    --uninstall|-u) ACTION="uninstall" ;;
    --help|-h)      usage; exit 0 ;;
    *)
      err "Unknown argument: $arg"
      usage
      exit 1
      ;;
  esac
done

# --- Preconditions -----------------------------------------------------------
if ! command -v cargo >/dev/null 2>&1; then
  err "cargo not found in PATH. Install Rust from https://rustup.rs/"
  exit 1
fi

if [[ ! -f "$SCRIPT_DIR/Cargo.toml" ]]; then
  err "Cargo.toml not found in $SCRIPT_DIR — run from project root."
  exit 1
fi

# --- Uninstall ---------------------------------------------------------------
if [[ "$ACTION" == "uninstall" ]]; then
  if [[ -e "$INSTALL_PATH" ]]; then
    log "Removing $INSTALL_PATH"
    rm -f "$INSTALL_PATH"
    log "Done."
  else
    log "$INSTALL_PATH not present — nothing to remove."
  fi
  exit 0
fi

# --- Install -----------------------------------------------------------------
log "Source:      $SCRIPT_DIR"
log "Install dir: $INSTALL_ROOT"
log "Binary:      $BINARY_NAME"
log "Building & installing (this may take a few minutes on first run)…"

# --locked: fail if Cargo.lock needs updating (matches CI / reproducible builds)
# --force:  overwrite any prior `fl` install
# --path:   install from local source (this directory)
cd "$SCRIPT_DIR"
cargo install --path "$SCRIPT_DIR" --locked --force

# --- Verify ------------------------------------------------------------------
if [[ -x "$INSTALL_PATH" ]]; then
  version_output="$("$INSTALL_PATH" --version 2>&1 || true)"
  log "Installed: $INSTALL_PATH"
  log "Version:   $version_output"
else
  err "cargo install reported success but $INSTALL_PATH is not executable."
  err "Check that the [[bin]] name in Cargo.toml matches: $BINARY_NAME"
  exit 1
fi

# --- PATH advisory -----------------------------------------------------------
case ":$PATH:" in
  *":$INSTALL_ROOT:"*) ;;
  *)
    warn ""
    warn "$INSTALL_ROOT is not in your PATH."
    warn "Add it to your shell rc file, e.g.:"
    warn "  export PATH=\"$INSTALL_ROOT:\$PATH\""
    ;;
esac
