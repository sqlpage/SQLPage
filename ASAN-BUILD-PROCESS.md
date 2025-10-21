# Building with ASAN for C Dependencies

## Two-Phase Build Approach

The solution uses a two-phase build to avoid proc-macro loading issues:

### Phase 1: Build proc-macros without ASAN
```bash
cargo build --lib --features odbc-static
```

This builds all libraries and proc-macros (like `sqlx-macros-oldapi`) with normal linking.

### Phase 2: Rebuild with ASAN instrumentation
```bash
# Set up ASAN environment
export CFLAGS="-fsanitize=address -fno-omit-frame-pointer -g"
export CXXFLAGS="-fsanitize=address -fno-omit-frame-pointer -g"
export CC=clang
export CXX=clang++
export RUSTFLAGS="-Clink-arg=-fsanitize=address"

# Preload ASAN runtime so rustc can load proc-macros
export LD_PRELOAD=$(clang -print-file-name=libclang_rt.asan-x86_64.so)

# Rebuild only C dependencies and main binary
cargo clean -p aws-lc-sys -p libsqlite3-sys -p zstd-sys -p sqlpage
cargo test --features odbc-static
```

## Why This Works

1. **Proc-macros** are built first without ASAN and cached
2. **C dependencies** (aws-lc-sys, libsqlite3-sys, zstd-sys) are recompiled with ASAN instrumentation
3. **LD_PRELOAD** loads the ASAN runtime library into the cargo/rustc process, allowing it to successfully load proc-macros even if they transitively depend on ASAN-instrumented code
4. **Final binary and tests** are linked with ASAN runtime via `-Clink-arg=-fsanitize=address`

## What Gets Detected

With ASAN enabled for C dependencies, the tests will detect:
- Buffer overflows in C code
- Use-after-free errors in C code
- Memory leaks in C code (via LeakSanitizer)
- Stack buffer overflows
- Global buffer overflows

This provides comprehensive memory safety checking for the native C dependencies while working around Rust's proc-macro system limitations.
