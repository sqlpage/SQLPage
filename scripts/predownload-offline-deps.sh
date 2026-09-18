#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "[1/2] Prefetching Cargo dependencies"
cargo fetch --locked

echo "[2/2] Prefetching npm dependencies"
npm ci --ignore-scripts --no-audit
if [ -f tests/end-to-end/package-lock.json ]; then
    (
        cd tests/end-to-end
        npm ci --ignore-scripts --no-audit
    )
fi

echo "Done. Offline caches are ready."
