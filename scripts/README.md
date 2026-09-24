# Scripts

## The Dockerfile runs these

- **`setup-cross-compilation.sh`** :: (1) Sets up the cross-compilation environment from the target and build architectures; (2) Installs system dependencies and cross-compilers; and (3) extracts libgcc for the runtime stage.
- **`build-dependencies.sh`** :: Builds only the dependencies for Docker layer caching.
- **`build-project.sh`**: Builds the SQLPage binary.
- **`build-frontend.mjs`**: Bundles the browser assets into `frontend/dist`. Also callable via `npm run build`.
- **`install-duckdb-odbc.sh`**: Installs the DuckDB ODBC driver into the `duckdb` image variant.
- **`setup-sqlpage-user.sh`**: Creates the unprivileged user the images run as.

These scripts pass configuration between build stages through temporary files in `/tmp/`.

## CI runs these

- **`package-test-binaries.sh`**: Compiles the Rust test harnesses once and tars them, so the database matrix runs the same executables instead of recompiling SQLPage six times.
- **`run-test-binaries.sh`**: Runs SQLPage compiled executables against whatever `DATABASE_URL` names.
- **`test-examples-hurl.sh`**: Starts an example's containers and runs its `test.hurl` suite. Takes an example path to filter on.
