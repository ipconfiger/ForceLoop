#!/bin/bash
set -euo pipefail

# publish-npm.sh — One-click build + publish all npm platform packages.
#
# Prerequisites:
#   1. cargo-zigbuild installed: cargo install cargo-zigbuild
#   2. npm logged in: npm login  (or NPM_TOKEN set in CI)
#   3. git tag pushed matching the version to publish
#
# Usage:
#   ./scripts/publish-npm.sh              # Build + publish all platforms
#   ./scripts/publish-npm.sh --publish     # Publish only (skip build)
#   ./scripts/publish-npm.sh --build       # Build + copy only (no publish)
#   ./scripts/publish-npm.sh --version     # Print current version and exit

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# ── Helpers ──────────────────────────────────────────────────────────

version() {
  python3 -c "import json; print(json.load(open('npm/main/package.json'))['version'])"
}

# Map target-triple → npm platform directory (case/esac for macOS bash 3.x)
target_to_npm_dir() {
  case "$1" in
    aarch64-apple-darwin)       echo "darwin-arm64"    ;;
    x86_64-apple-darwin)        echo "darwin-x64"      ;;
    x86_64-unknown-linux-gnu)   echo "linux-x64"       ;;
    aarch64-unknown-linux-gnu)  echo "linux-arm64"     ;;
    x86_64-unknown-linux-musl)  echo "linux-x64-musl"  ;;
    aarch64-unknown-linux-musl) echo "linux-arm64-musl" ;;
    x86_64-pc-windows-gnu)      echo "win32-x64"       ;;
    *)                          echo ""                ;;
  esac
}

ALL_TARGETS="aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-gnu"

# ── Build all platform binaries via cargo-zigbuild ──────────────────

build_all() {
  echo "═══ Building all platform binaries ═══"
  echo ""

  # Detect native target — use regular cargo, zigbuild fails on macOS
  local native_target
  native_target="$(rustc -vV | grep host | awk '{print $2}')"

  for target in $ALL_TARGETS; do
    npm_dir="$(target_to_npm_dir "$target")"
    echo "▸ $target → npm/$npm_dir"

    if [ "$target" = "$native_target" ]; then
      cargo build --release 2>&1 | tail -1
    else
      cargo zigbuild --release --target "$target" 2>&1 | tail -1
    fi
  done
  echo ""
  echo "✅ All builds complete"
}

# ── Copy binaries into npm platform directories ─────────────────────

copy_binaries() {
  echo ""
  echo "═══ Copying binaries ═══"
  local native_target
  native_target="$(rustc -vV | grep host | awk '{print $2}')"

  for target in $ALL_TARGETS; do
    npm_dir="$(target_to_npm_dir "$target")"

    # Binary location: native targets go to target/release/, cross targets to target/<triple>/release/
    local search_dirs
    if [ "$target" = "$native_target" ]; then
      search_dirs="$CARGO_TARGET_DIR/release $PROJECT_ROOT/target/release"
    else
      search_dirs="$CARGO_TARGET_DIR/$target/release $PROJECT_ROOT/target/$target/release"
    fi

    # Determine binary source path
    local src=""
    for dir in $search_dirs; do
      if [ -f "$dir/fl.exe" ]; then src="$dir/fl.exe"; break; fi
      if [ -f "$dir/fl" ]; then src="$dir/fl"; break; fi
    done

    if [ -z "$src" ]; then
      echo "  ⚠ Binary not found for $target — skipping"
      continue
    fi

    local dest_dir="npm/$npm_dir/package"
    mkdir -p "$dest_dir"

    if [[ "$target" == *"-windows-"* ]]; then
      cp "$src" "$dest_dir/fl.exe"
      chmod +x "$dest_dir/fl.exe" 2>/dev/null || true
      local size; size="$(ls -lh "$dest_dir/fl.exe" | awk '{print $5}')"
      echo "  → $dest_dir/fl.exe ($size)"
    else
      cp "$src" "$dest_dir/fl"
      chmod +x "$dest_dir/fl"
      local size; size="$(ls -lh "$dest_dir/fl" | awk '{print $5}')"
      echo "  → $dest_dir/fl ($size)"
    fi

    # Update platform package version
    python3 -c "
import json
d = json.load(open('npm/$npm_dir/package.json'))
d['version'] = '$ver'
json.dump(d, open('npm/$npm_dir/package.json', 'w'), indent=2)
"
  done

  # Update main package version + optionalDependencies
  python3 -c "
import json
d = json.load(open('npm/main/package.json'))
d['version'] = '$ver'
for k in d.get('optionalDependencies', {}):
    d['optionalDependencies'][k] = '$ver'
json.dump(d, open('npm/main/package.json', 'w'), indent=2)
"
  echo "  → npm/main/package.json (version $ver)"
  echo ""
  echo "✅ All binaries copied (version $ver)"
}

# ── Publish all platform packages then main ─────────────────────────

publish_all() {
  echo ""
  echo "═══ Publishing to npm ═══"
  local ver; ver="$(version)"

  # Publish platform packages first
  for target in $ALL_TARGETS; do
    npm_dir="$(target_to_npm_dir "$target")"
    echo ""
    echo "▸ Publishing @forceloop/cli-$npm_dir@$ver ..."

    cd "npm/$npm_dir"
    if output="$(npm publish --access public 2>&1)"; then
      echo "  ✅ Published"
    elif echo "$output" | grep -q "You cannot publish over"; then
      echo "  ℹ Already published — skipped"
    elif echo "$output" | grep -q "2FA\|one-time password\|otp"; then
      echo "  ⚠ Need OTP — run with --otp or login interactively"
      echo "     cd npm/$npm_dir && npm publish --access public"
    else
      echo "  ⚠ Failed: $(echo "$output" | tail -1)"
    fi
    cd "$PROJECT_ROOT"
    sleep 8  # Avoid npm 409 race
  done

  # Publish main package
  echo ""
  echo "▸ Publishing @forceloop/cli@$ver ..."
  cd "npm/main"
  if output="$(npm publish --access public 2>&1)"; then
    echo "  ✅ Published"
  elif echo "$output" | grep -q "You cannot publish over"; then
    echo "  ℹ Already published — skipped"
  elif echo "$output" | grep -q "2FA\|one-time password\|otp"; then
    echo "  ⚠ Need OTP — run with --otp or login interactively"
    echo "     cd npm/main && npm publish --access public"
  else
    echo "  ⚠ Failed: $(echo "$output" | tail -1)"
  fi
  cd "$PROJECT_ROOT"

  echo ""
  echo "🎉 All done! https://www.npmjs.com/package/@forceloop/cli"
}

# ── Main ─────────────────────────────────────────────────────────────

case "${1:-all}" in
  --version|-v)
    version
    ;;
  --publish|-p)
    copy_binaries
    publish_all
    ;;
  --build|-b)
    build_all
    copy_binaries
    ;;
  all|"")
    build_all
    copy_binaries
    publish_all
    ;;
  *)
    echo "Usage: $0 [--build | --publish | all | --version]"
    echo ""
    echo "  all        Build + copy + publish (default)"
    echo "  --build    Build + copy only (no publish)"
    echo "  --publish  Copy + publish only (skip build)"
    echo "  --version  Show current version"
    exit 1
    ;;
esac