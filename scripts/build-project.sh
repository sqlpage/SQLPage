#!/bin/bash
set -euo pipefail

source /tmp/build-env.sh

echo "Building project for target: $TARGET"

cargo build \
    --target "$TARGET" \
    --config "target.$TARGET.linker=\"$LINKER\"" \
    --features odbc-static \
    --profile superoptimized

mv "target/$TARGET/superoptimized/sqlpage" sqlpage.bin
