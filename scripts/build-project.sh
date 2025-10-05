#!/bin/bash
set -euo pipefail

# Source the environment variables set by setup-cross-compilation.sh
TARGET="$(cat /tmp/TARGET)"
LINKER="$(cat /tmp/LINKER)"

echo "Building project for target: $TARGET"

# Build the project
touch src/main.rs
cargo build \
    --target "$TARGET" \
    --config "target.$TARGET.linker=\"$LINKER\"" \
    --features odbc-static \
    --profile superoptimized

# Move the binary to the expected location
mv "target/$TARGET/superoptimized/sqlpage" sqlpage.bin
