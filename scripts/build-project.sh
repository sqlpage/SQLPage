#!/bin/bash
set -euo pipefail

source /tmp/build-env.sh

PROFILE="${CARGO_PROFILE:-superoptimized}"
echo "Building project for target: $TARGET (profile: $PROFILE)"

cargo build \
    --target "$TARGET" \
    --config "target.$TARGET.linker=\"$LINKER\"" \
    --features odbc-static \
    --profile "$PROFILE"

mv "target/$TARGET/$PROFILE/sqlpage" sqlpage.bin
