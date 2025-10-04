Core Concept: User writes .sql files, SQLPage executes queries, results mapped to handlebars UI components,
HTML streamed to client

## Validation

### When working on rust code
Mandatory formatting (rust): `cargo fmt --all`
Mandatory linting: `cargo clippy --all-targets --all-features -- -D warnings`

### When working on css or js
Frontend formatting: `npm run format`

More about testing: see [github actions](./.github/workflows/ci.yml).
Project structure: see [contribution guide](./CONTRIBUTING.md)

NEVER reformat/lint/touch files unrelated to your task. Always run tests/lints/format before stopping when you changed code.

### Testing

```
cargo test # tests with inmemory sqlite by default
```

For other databases, see [docker testing setup](./docker-compose.yml)

```
docker compose up -d mssql # or postgres or mysql
DATABASE_URL='mssql://root:Password123!@localhost/sqlpage' cargo test # all dbms use the same user:pass and db name
```

#### Project Conventions

- Components: defined in `./sqlpage/templates/*.handlebars`
- Functions: `src/webserver/database/sqlpage_functions/functions.rs` registered with `make_function!`.
- Components and functions are documented in [official website](./examples/official-site/sqlpage/migrations/); one migration per component and per function.
  - tables
    - `component(name,description,icon,introduced_in_version)` -- icon name from tabler icon
    - `parameter(top_level BOOLEAN, name, component REFERENCES component(name), description, description_md, type, optional BOOLEAN)` parameter types: BOOLEAN, COLOR, HTML, ICON, INTEGER, JSON, REAL, TEXT, TIMESTAMP, URL
    - `example(component REFERENCES component(name), description, properties JSON)`
- [Configuration](./configuration.md): see [AppConfig](./src/app_config.rs)
- Routing: file-based in `src/webserver/routing.rs`; not found handled via `src/default_404.sql`.
- Follow patterns from similar modules before introducing new abstractions.
- frontend: see [css](./sqlpage/sqlpage.css) and [js](./sqlpage/sqlpage.js)