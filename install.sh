#!/usr/bin/env bash
# install.sh — Install `fl` (ForceLoop) from this directory via `cargo install`.
set -euo pipefail
cd "$(dirname "$0")"
exec cargo install --path . --locked --force
