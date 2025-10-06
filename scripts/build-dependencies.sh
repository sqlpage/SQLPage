#!/bin/bash
set -euo pipefail

# Source the environment variables set by setup-cross-compilation.sh
TARGET="$(cat /tmp/TARGET)"
LINKER="$(cat /tmp/LINKER)"
BINDGEN_EXTRA_CLANG_ARGS="$(cat /tmp/BINDGEN_EXTRA_CLANG_ARGS || true)"

echo "Building dependencies for target: $TARGET"

# Build dependencies only (for Docker layer caching)
cargo build \
    --target "$TARGET" \
    --config "target.$TARGET.linker=\"$LINKER\"" \
    --features odbc-static \
    --profile superoptimized
