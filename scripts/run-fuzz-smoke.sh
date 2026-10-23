#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

corpus_dir="$(mktemp -d)"
trap 'rm -rf "$corpus_dir"' EXIT

python3 - "$corpus_dir" <<'PY'
import sys
from pathlib import Path

corpus = Path(sys.argv[1])
for index, source in enumerate(sorted(Path("tests/sql_test_files").rglob("*.sql"))):
    for selector in range(16):
        (corpus / f"{index}-{selector}").write_bytes(
            bytes([selector]) + source.read_bytes()
        )
PY

toolchain="${FUZZ_TOOLCHAIN:-nightly}"
cargo "+${toolchain}" fuzz run --dev sql_file "$corpus_dir" -- \
  -runs=4096 \
  -seed=1 \
  -max_len=16384 \
  -timeout=5
