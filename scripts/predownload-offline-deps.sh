#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "[1/3] Prefetching Cargo dependencies"
cargo fetch --locked

echo "[2/3] Prefetching npm dependencies"
npm ci --ignore-scripts --no-audit

echo "[3/3] Building the frontend assets the Rust build embeds"
npm run build

echo "Done. Offline caches are ready."
