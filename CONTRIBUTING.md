# Contributing to SQLPage

Thank you for your interest in contributing to SQLPage! This document will guide you through the contribution process.

## Development Setup

1. Install Rust and Cargo (latest stable version): https://www.rust-lang.org/tools/install
2. Install Node.js: https://nodejs.org/en/download/
3. Clone the repository

```bash
git clone https://github.com/sqlpage/sqlpage
cd sqlpage
```

## Building the project

The first time you build the project, Cargo and npm download their
dependencies, so you will need internet access, and the build may take a while.

Run this command from the root of a checkout to install the locked npm dependencies, build the browser assets, and build SQLPage in development mode:

```bash
npm run build:rust
```

After the browser assets have been built, you can run `cargo build` directly for Rust-only changes. Run `npm run build` again after changing frontend sources. Builds from the published crate use its included browser assets and do not need Node.js.

The resulting executable will be in `target/debug/sqlpage`.

### Release mode

To build the project in release mode:

```bash
npm run build:rust -- --release
```

The resulting executable will be in `target/release/sqlpage`.

### ODBC build modes

SQLPage can either be built with an integrated odbc driver manager (static linking),
or depend on having one already installed on the system where it is running (dynamic linking).

- Dynamic ODBC (default): `npm run build:rust`
- Static ODBC (Linux and MacOS only): `npm run build:rust -- --features odbc-static`

Windows comes with ODBC pre-installed; SQLPage cannot statically link to the unixODBC driver manager on windows.

## Code Style and Linting

### Rust

- Use `cargo fmt --all` to format your Rust code
- Run `cargo clippy` to catch common mistakes and improve code quality
- All code must pass the following checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Frontend

We use Biome for linting and formatting of the frontend code, and TypeScript
to typecheck it.

```bash
npm install # once
npm run format # apply formatting
npm test # the check CI runs: biome, typecheck, and the frontend unit tests
```

`npm test` checks the entire frontend codebase (html, css, js, ts).

## Testing

### Rust Tests

Run the backend tests:

```bash
cargo test
```

By default, tests use in-memory SQLite. An exported `DATABASE_URL` overrides this default;
unset it when returning to SQLite.

The Linux [CI database matrix](./.github/workflows/ci.yml) runs the same Rust test binaries against:

| Database | Connection path | Local server/driver setup |
| --- | --- | --- |
| SQLite | Native | In-memory; no server required |
| PostgreSQL | Native | `docker compose up --wait postgres` |
| MySQL | Native | `docker compose up --wait mysql` |
| Microsoft SQL Server | Native | `docker compose up --wait mssql` |
| Oracle | ODBC | `docker compose up --wait oracle` and an Oracle ODBC driver on the host |
| DuckDB | ODBC | A DuckDB ODBC driver on the host; no server required |

Windows also runs `cargo test` with the default SQLite database. MariaDB is available in
`docker-compose.yml` for additional local testing, but is not a separate CI matrix entry.

For example, run the backend tests against SQL Server:

```bash
docker compose up --wait mssql
DATABASE_URL='mssql://root:Password123!@localhost/sqlpage' cargo test
```

On Linux and macOS, `cargo test --features odbc-static` matches CI's static unixODBC linking.
This bundles the driver manager, **not** the database drivers. Oracle and DuckDB still need their
own installed drivers. Use the connection strings and driver setup in the CI matrix, including
Oracle's `LD_LIBRARY_PATH` and `ODBCSYSINI` environment settings. See
[`scripts/install-oracle-odbc.sh`](./scripts/install-oracle-odbc.sh) and
[`scripts/install-duckdb-odbc.sh`](./scripts/install-duckdb-odbc.sh) for the CI installation steps.

A SQLite pass does not establish portability. For SQL execution, binding, or result decoding
changes, check the affected database matrix jobs before considering the change verified.
Distinguish an assertion failure from a driver/setup error or a timeout after the test summary;
a passing summary alone does not mean the test process exited successfully.

#### Writing SQL regression tests

Extend existing fixtures and parameterize shared setup before adding a new harness. Keep distinct
assertions for each behavior and failure mode; consolidate repetition without dropping scenarios.
Shared SQL fixtures live in [`tests/sql_test_files/`](./tests/sql_test_files/); database-specific
syntax belongs under `data/database-specific/<database>/`. Reuse the request and transaction suites
for precedence and rollback checks, and the common one-connection configuration for nested queries.

Keep these differences in mind:

- **Numbers inside JSON strings:** Oracle ODBC can serialize a selected numeric literal as `1.0`
  while another driver or a JSON constructor produces `1`. A JSON string returned by
  `sqlpage.run_sql` is compared as a string by the fixture harness, so these differ. Use text
  markers when testing row order, or an explicit numeric comparison when testing numeric values;
  do not weaken full-row assertions to substring checks to hide formatting differences.
- **Scalar `SET` queries:** Zero rows produce NULL. SQLite takes the first row when there are
  several; other supported engines reject multiple rows. Each returned scalar row must have exactly
  one output column, even when duplicate names would merge into one JSON property. Zero-row
  queries return NULL without checking a row's column count. Keep engine-specific
  cardinality expectations in the existing database-specific fixtures.
- **Variables:** The current implementation resolves `$name` through SET then GET, and `:name`
  through SET then POST. With no SET value, a matching POST field triggers a `$name` warning;
  lookup still does not fall back to POST.
  An explicitly NULL SET value stops lookup rather than falling through to request inputs.
  `sqlpage.variables()` has a different merged enumeration policy: SET > POST > GET. Test lookup
  and enumeration separately, and check that `run_sql` child assignments do not change the parent.
- **Rows and computed values:** Duplicate physical output names accumulate values in fetched
  column order; SQLPage-computed columns append afterwards, regardless of SELECT order.
  For example, `select sqlpage.url_encode(name) as encoded, name as encoded` returns the physical
  `name` value before the encoded value. Private function-input columns must stay out of response JSON. Nested
  `run_sql` evaluation must release the active fetch stream before reusing its connection. Keep
  the one-connection pool setting in regression fixtures so ownership mistakes cannot be hidden
  by acquiring another connection.
- **Fixture conventions:** In JSON-mode tests, return `actual` and `expected`; an `expected` array
  lists acceptable alternatives, not an expected row sequence. Compare a complete JSON string
  when asserting row sequence. For HTML error fixtures, `error_<message>.sql` supplies the expected
  message (underscores become spaces). Use `_no<database>` exclusions only for an established
  backend limitation, with a comment explaining it.

### End-to-End Tests

We use Playwright for end-to-end testing of dynamic frontend features.
Tests are located in [`tests/end-to-end/`](./tests/end-to-end/). Key areas covered include:

Component tests use deterministic SQL applications under `tests/end-to-end/fixtures/<suite>/`.
Each suite contains an `index.sql` page and a `test.ts` file. Import `test` and `expect` from
`../../fixture`; the shared fixture opens the matching SQL page before every test and waits for
all SQLPage components to initialize. Prefer role, label, and text locators over CSS selectors
when the assertion does not specifically concern generated markup.

Keep official-site smoke and integration tests in root-level `*.spec.ts` files. Component behavior
tests should render real components through their SQL fixture; do not inject component markup or
invoke SQLPage's JavaScript initialization functions directly. Parameterized fixtures may accept
request variables when several tests need the same component with different data.

#### Run the tests

```bash
npm install
cd tests/end-to-end
npx playwright install chromium
npm run test
```

Playwright starts both servers itself on a free port. Set `SQLPAGE_BINARY` to run the servers from an already compiled binary instead of `cargo run`.

## Documentation

### Component Documentation

When adding new components, comprehensive documentation is required. Example from a component documentation:

```sql
INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('component_name', 'icon_name', 'Description of the component', 'version');

-- Document all parameters
INSERT INTO parameter(component, name, description, type, top_level, optional)
VALUES ('component_name', 'param_name', 'param_description', 'TEXT|BOOLEAN|INTEGER|JSON|ICON|COLOR|HTML|REAL|TIMESTAMP|URL', false, true);

-- Use description_md instead of description when the text contains markdown
INSERT INTO parameter(component, name, description_md, type, top_level, optional)
VALUES ('component_name', 'other_param', 'Set to `true` to see [the docs](/documentation.sql).', 'BOOLEAN', true, true);

-- Include usage examples
INSERT INTO example(component, description, properties) VALUES
    ('component_name', 'Example description in markdown', JSON('[
{"component": "new_component_name", "top_level_property_1": "value1", "top_level_property_2": "value2"},
{"row_level_property_1": "value1", "row_level_property_2": "value2"}
]'));
```

Component documentation is stored in [`./examples/official-site/sqlpage/migrations/`](./examples/official-site/sqlpage/migrations/).

If you are editing an existing component, edit the existing sql documentation file directly.
If you are adding a new component, add a new sql file in the folder, and add the appropriate insert statements above.

### SQLPage Function Documentation

When adding new SQLPage functions, document them using a SQL migrations. Example structure:

```sql
-- Function Definition
INSERT INTO sqlpage_functions (
    "name",
    "introduced_in_version",
    "icon",
    "description_md"
)
VALUES (
    'your_function_name',
    '1.0.0',
    'function-icon-name',
    'Description of what the function does.

### Example

    select ''text'' as component, sqlpage.your_function_name(''parameter'') as result;

Additional markdown documentation, usage notes, and examples go here.
');

-- Function Parameters
INSERT INTO sqlpage_function_parameters (
    "function",
    "index",
    "name",
    "description_md",
    "type"
)
VALUES (
    'your_function_name',
    1,
    'parameter_name',
    'Description of what this parameter does and how to use it.',
    'TEXT|BOOLEAN|INTEGER|JSON'
);
```

Key elements to include in function documentation:

- Clear description of the function's purpose
- Version number where the function was introduced
- Appropriate icon
- Markdown-formatted documentation with examples
- All parameters documented with clear descriptions and types
- Security considerations if applicable
- Example usage scenarios

## Pull Request Process

1. Create a new branch for your feature/fix:

```bash
git checkout -b feature/your-feature-name
```

2. Make your changes, ensuring:

- All tests pass
- Code is properly formatted
- New features are documented
- tests cover new functionality
- `CHANGELOG.md` has an entry for any user-visible change

3. Push your changes and create a Pull Request

4. CI Checks
   Our CI pipeline will automatically:
   - Run Rust formatting and clippy checks
   - Execute all tests across multiple platforms (Linux, Windows)
   - Build Docker images for multiple architectures
   - Run frontend linting, typechecking and unit tests (`npm test`)
   - Test against SQLite, PostgreSQL, MySQL, Microsoft SQL Server, Oracle via ODBC, and DuckDB via ODBC

## Release Process

Releases are automated when pushing tags that match the pattern `v*` (e.g., `v1.0.0`). The CI pipeline will:

- Build and test the code
- Create Docker images for multiple architectures
- Push images to Docker Hub
- Create GitHub releases

## Questions?

If you have any questions, feel free to open an issue or discussion on GitHub.
