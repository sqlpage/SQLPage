# SQL file fuzzing

`sql_file` exercises the production tokenizer, parser, SQLPage statement and
expression classification, projection partitioning, parameter binding, backend
lowering, and error diagnostics. The first input byte selects one of eight
database dialects and native/ODBC placeholder handling; the rest is UTF-8 SQL.
Malformed SQL is expected. Panics, sanitizer failures, and hangs are failures.
Compilation is synchronous and does not execute SQL or SQLPage functions.

Build browser assets as described in [CONTRIBUTING.md](../CONTRIBUTING.md), then:

```sh
cargo install cargo-fuzz --locked
rustup toolchain install nightly
cargo +nightly fuzz run sql_file -- -max_len=16384
```

Seed the corpus from existing regression fixtures, including every dialect:

```sh
python3 - <<'PY'
from pathlib import Path

corpus = Path("fuzz/corpus/sql_file")
corpus.mkdir(parents=True, exist_ok=True)
for index, source in enumerate(sorted(Path("tests/sql_test_files").rglob("*.sql"))):
    for selector in range(16):
        (corpus / f"{index}-{selector}").write_bytes(bytes([selector]) + source.read_bytes())
PY
```

The 16 KiB limit is a starting point for efficient mutation, not a parser limit.
The harness also accepts larger inputs. The same cargo-fuzz target is used by
OSS-Fuzz and by OSS Verify's static RISC-V adapter.

To run the bounded, deterministic SQL fixture fuzz test used by CI:

```sh
scripts/test-sql-fixture-fuzz.sh
```
