#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

BINARY_DIR=target/sqlpage-test-binaries
ARCHIVE=target/sqlpage-linux-test-binaries.tar.gz

rm -rf "$BINARY_DIR"
mkdir -p "$BINARY_DIR"

cargo test --features odbc-static --no-run --message-format=json \
  | jq -r 'select(.profile.test == true and .executable != null) | .executable' \
  | while IFS= read -r test_binary; do
      cp -- "$test_binary" "$BINARY_DIR/"
    done

test -n "$(find "$BINARY_DIR" -maxdepth 1 -type f -print -quit)"
tar -C "$BINARY_DIR" -czf "$ARCHIVE" .
