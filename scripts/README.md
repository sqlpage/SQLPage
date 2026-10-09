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
- **`install-oracle-odbc.sh`**: Verifies and extracts the pinned Oracle Instant Client 23ai ZIPs, registers a private ODBC driver, and writes its environment to `GITHUB_ENV`. CI caches the archives under `RUNNER_TEMP`. Requires curl, unzip, unixODBC, and libaio.
- **`test-examples-hurl.sh`**: Starts an example's containers, retries only the first safe GET served by SQLPage as a readiness check, then runs the suite without global retries. Takes an example path to filter on; Mailpit cleanup and telemetry trace lookups retain per-request retries for service readiness and Tempo indexing.
