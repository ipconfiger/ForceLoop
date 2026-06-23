#!/bin/bash
set -euo pipefail

# build-npm.sh — Build the Rust binary and package it into npm platform directories.
#
# Usage:
#   ./scripts/build-npm.sh                         # Build for current platform
#   ./scripts/build-npm.sh <target-triple>          # Cross-compile for specific target
#   ./scripts/build-npm.sh all                      # Build for all platforms (CI)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Get the actual cargo target directory (respects CARGO_TARGET_DIR / CARGO_BUILD_TARGET_DIR env vars)
CARGO_TARGET_DIR="$(
  cargo metadata --format-version=1 --no-deps 2>/dev/null \
    | grep -o '"target_directory":"[^"]*"' \
    | cut -d'"' -f4
)"

build_and_package() {
  local target="$1"
  local npm_dir="$2"
  shift 2

  echo "==> Building for $target (→ npm/$npm_dir)"

  if [ -z "${DO_NOT_BUILD:-}" ]; then
    if [ "$target" = "native" ]; then
      cargo build --release
    else
      rustup target add "$target" 2>/dev/null || true
      cargo build --release --target "$target"
    fi
  fi

  # Determine binary source path
  local binary_src
  if [ "$target" = "native" ]; then
    binary_src="$CARGO_TARGET_DIR/release/fl"
  else
    binary_src="$CARGO_TARGET_DIR/$target/release/fl"
  fi

  # Determine binary destination path
  if echo "$target" | grep -q "windows"; then
    binary_src="${binary_src}.exe"
    local dest="$PROJECT_ROOT/npm/$npm_dir/package/fl.exe"
  else
    local dest="$PROJECT_ROOT/npm/$npm_dir/package/fl"
  fi

  mkdir -p "$PROJECT_ROOT/npm/$npm_dir/package"
  cp "$binary_src" "$dest"
  chmod +x "$dest" 2>/dev/null || true

  echo "  → $dest ($(ls -lh "$dest" | awk '{print $5}'))"
}

cd "$PROJECT_ROOT"

case "${1:-native}" in
  native)
    local_target="$(rustc -vV | grep host | awk '{print $2}')"
    case "$local_target" in
      aarch64-apple-darwin)   npm_dir="darwin-arm64" ;;
      x86_64-apple-darwin)    npm_dir="darwin-x64" ;;
      x86_64-unknown-linux-gnu) npm_dir="linux-x64" ;;
      aarch64-unknown-linux-gnu) npm_dir="linux-arm64" ;;
      x86_64-unknown-linux-musl) npm_dir="linux-x64-musl" ;;
      aarch64-unknown-linux-musl) npm_dir="linux-arm64-musl" ;;
      x86_64-pc-windows-msvc) npm_dir="win32-x64" ;;
      aarch64-pc-windows-msvc) npm_dir="win32-arm64" ;;
      *)
        echo "Unsupported platform: $local_target"
        exit 1
        ;;
    esac
    build_and_package "native" "$npm_dir"
    ;;

  all)
    build_and_package "aarch64-apple-darwin" "darwin-arm64"
    build_and_package "x86_64-apple-darwin" "darwin-x64"
    build_and_package "x86_64-unknown-linux-gnu" "linux-x64"
    build_and_package "aarch64-unknown-linux-gnu" "linux-arm64"
    build_and_package "x86_64-unknown-linux-musl" "linux-x64-musl"
    build_and_package "aarch64-unknown-linux-musl" "linux-arm64-musl"
    build_and_package "x86_64-pc-windows-msvc" "win32-x64"
    build_and_package "aarch64-pc-windows-msvc" "win32-arm64"
    ;;

  *)
    # Single target triple
    target="${1}"
    case "$target" in
      aarch64-apple-darwin)   npm_dir="darwin-arm64" ;;
      x86_64-apple-darwin)    npm_dir="darwin-x64" ;;
      x86_64-unknown-linux-gnu) npm_dir="linux-x64" ;;
      aarch64-unknown-linux-gnu) npm_dir="linux-arm64" ;;
      x86_64-unknown-linux-musl) npm_dir="linux-x64-musl" ;;
      aarch64-unknown-linux-musl) npm_dir="linux-arm64-musl" ;;
      x86_64-pc-windows-msvc) npm_dir="win32-x64" ;;
      aarch64-pc-windows-msvc) npm_dir="win32-arm64" ;;
      *)
        echo "Unknown target: $1"
        exit 1
        ;;
    esac
    build_and_package "$target" "$npm_dir"
    ;;
esac

echo ""
echo "Done! Package the platform tarball with:"
echo "  cd npm/<platform> && npm pack"
echo ""
echo "Package the main tarball with:"
echo "  cd npm/main && npm pack"