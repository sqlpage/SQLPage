# Contributing to SQLPage

Thank you for your interest in contributing to SQLPage! This document will guide you through the contribution process.

## Development Setup

1. Install Rust and Cargo (latest stable version): https://www.rust-lang.org/tools/install
2. If you contribute to the frontend, install Node.js too for frontend tooling: https://nodejs.org/en/download/
3. Clone the repository

```bash
git clone https://github.com/sqlpage/sqlpage
cd sqlpage
```

## Code Style and Linting

### Rust
- Use `cargo fmt` to format your Rust code
- Run `cargo clippy` to catch common mistakes and improve code quality
- All code must pass the following checks:
```bash
cargo fmt --all -- --check
cargo clippy
```

### Frontend

We use Biome for linting and formatting of the frontend code.

```bash
npx @biomejs/biome check .
```
This will check the entire codebase (html, css, js).

## Testing

### Rust Tests

Run the backend tests:

```bash
cargo test
```

By default, the tests are run against an SQLite in-memory database.

If you want to run them against another database,
start a database server with `docker compose up database_name` (mssql, mysql, mariadb, or postgres)
and run the tests with the `DATABASE_URL` environment variable pointing to the database:

```bash
docker compose up mssql # or mysql, mariadb, postgres
export DATABASE_URL=mssql://root:Password123!@localhost/sqlpage
cargo test
```

### End-to-End Tests
We use Playwright for end-to-end testing of dynamic frontend features.
Tests are located in [`tests/end-to-end/`](./tests/end-to-end/). Key areas covered include:

#### Start a sqlpage instance pointed to the official site source code

```bash
cd examples/official-site
cargo run
```

#### Run the tests

In a separate terminal, run the tests:

```bash
cd tests/end-to-end
npm install
npm run test
```

## Documentation

### Component Documentation
When adding new components, comprehensive documentation is required. Example from a component documentation:

```sql
INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('component_name', 'icon_name', 'Description of the component', 'version');

-- Document all parameters
INSERT INTO parameter(component, name, description, type, top_level, optional) 
VALUES ('component_name', 'param_name', 'param_description', 'TEXT|BOOLEAN|NUMBER|JSON|ICON|COLOR', false, true);

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
    'TEXT|BOOLEAN|NUMBER|JSON'
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

3. Push your changes and create a Pull Request

4. CI Checks
   Our CI pipeline will automatically:
   - Run Rust formatting and clippy checks
   - Execute all tests across multiple platforms (Linux, Windows)
   - Build Docker images for multiple architectures
   - Run frontend linting with Biome
   - Test against multiple databases (PostgreSQL, MySQL, MSSQL)

5. Wait for review and address any feedback

## Release Process

Releases are automated when pushing tags that match the pattern `v*` (e.g., `v1.0.0`). The CI pipeline will:
- Build and test the code
- Create Docker images for multiple architectures
- Push images to Docker Hub
- Create GitHub releases

## Questions?

If you have any questions, feel free to open an issue or discussion on GitHub.
