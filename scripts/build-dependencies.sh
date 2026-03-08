#!/bin/bash
set -euo pipefail

source /tmp/build-env.sh

PROFILE="${CARGO_PROFILE:-superoptimized}"
echo "Building dependencies for target: $TARGET (profile: $PROFILE)"

cargo build \
    --target "$TARGET" \
    --config "target.$TARGET.linker=\"$LINKER\"" \
    --features odbc-static \
    --profile "$PROFILE"
