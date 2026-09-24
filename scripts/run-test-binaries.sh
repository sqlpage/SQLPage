#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

shopt -s nullglob
test_binaries=(target/sqlpage-test-binaries/*)

if ((${#test_binaries[@]} == 0)); then
  echo "No test binaries were found in target/sqlpage-test-binaries" >&2
  exit 1
fi

for test_binary in "${test_binaries[@]}"; do
  echo "::group::$(basename "$test_binary")"
  "$test_binary" --quiet
  echo "::endgroup::"
done
