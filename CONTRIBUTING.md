# Contributing to SQLPage

## Setup and build

Install stable Rust and Node.js (see [CI](./.github/actions/install-frontend-dependencies/action.yml)
for the Node version), then run this command from a checkout:

```bash
npm run build:rust
```

This installs locked npm dependencies, builds browser assets, and creates
`target/debug/sqlpage`. Afterwards, use `cargo build` for Rust changes and
`npm run build` for frontend changes. Add `-- --release` to `npm run build:rust`
for a release build.

Default builds require a system ODBC driver manager. On Linux and macOS,
`--features odbc-static` bundles unixODBC, but database-specific drivers must still
be installed separately. Windows uses its system ODBC driver manager. Published
crates include browser assets and do not require Node.js.

## Validation

For Rust changes:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

For frontend changes:

```bash
npm run format
npm test
npm run build
```

Keep formatting scoped to changed files. `npm test` checks formatting, lint,
types, and frontend unit tests. Dynamic frontend changes also need Playwright:

```bash
cd tests/end-to-end
npx playwright install --with-deps --only-shell chromium
npm test
```

Playwright starts its servers on free ports. Set `SQLPAGE_BINARY` to use an already
built executable. Component suites use `fixtures/<suite>/{index.sql,test.ts}` and
import the shared `fixture.ts` harness; root-level `*.spec.ts` files cover the
official site. For examples with `test.hurl`, run
`scripts/test-examples-hurl.sh '<example-path>'` from the repository root.

Integration tests use the shared request helpers in [tests/common](./tests/common/mod.rs)
to exercise production routes and middleware and drain request pools.

Rust tests default to in-memory SQLite; `DATABASE_URL` selects another database.
For example:

```bash
docker compose up --wait mssql
DATABASE_URL='mssql://root:Password123!@localhost/sqlpage' cargo test
```

Use the [CI matrix](./.github/workflows/ci.yml) for supported backends, connection
strings, and driver setup. On Linux and macOS, `cargo test --features odbc-static`
matches CI's driver-manager linking. Oracle and DuckDB need host ODBC drivers even
when the database runs in a container. For local Oracle runs, add
`-- --test-threads=2` to avoid overwhelming the listener. Check affected matrix results for SQL
execution changes; SQLite alone cannot establish portability.

## Submitting a change

Follow nearby code and tests, add regression coverage, and describe the resulting
behavior and validation in the pull request. Keep changes and formatting focused.
Architectural constraints and documentation maintenance rules are in
[AGENTS.md](./AGENTS.md).

User documentation lives in [the official-site SQL pages and migrations](./examples/official-site/).
Edit existing documentation in place: deployment recreates the site database.
Use nearby entries as examples rather than copying schemas into this guide. Update
[configuration.md](./configuration.md) for configuration changes and
[CHANGELOG.md](./CHANGELOG.md) for user-visible changes in an unreleased version.
