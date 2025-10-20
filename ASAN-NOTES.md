# Address Sanitizer (ASAN) Configuration

## C Dependencies with ASAN

This project successfully compiles C dependencies with ASAN enabled:
- aws-lc-sys (AWS-LC crypto library)
- libsqlite3-sys (SQLite)
- zstd-sys (Zstandard compression)  
- unix-odbc (when using odbc-static feature)

### Configuration

The C dependencies are compiled with ASAN using these environment variables:
```bash
export CFLAGS="-fsanitize=address -fno-omit-frame-pointer -g"
export CXXFLAGS="-fsanitize=address -fno-omit-frame-pointer -g"
export CC=clang
export CXX=clang++
```

A custom linker wrapper ensures ASAN runtime is linked:
```bash
#!/bin/bash
exec clang -fsanitize=address "$@"
```

### Known Limitation: Proc-Macros with `-Zbuild-std`

Enabling full Rust ASAN (`-Zsanitizer=address`) with `-Zbuild-std` causes issues with proc-macro crates.
The sqlx-oldapi dependency uses sqlx-macros-oldapi (a proc-macro), which cannot be loaded when 
the standard library is rebuilt with ASAN.

## Workaround for Testing

For CI/testing purposes, we can:
1. Compile C dependencies with ASAN only (catches memory issues in native code)
2. Run tests with ASAN enabled to detect memory errors
3. Use LeakSanitizer (LSAN) in addition to catch memory leaks

This provides significant value as the C dependencies are where most memory safety issues occur.
