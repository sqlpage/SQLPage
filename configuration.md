# Configuring SQLPage

SQLPage can be configured through either environment variables or a JSON file placed at `sqlpage/sqlpage.json`.

The complete, generated reference for every configuration option, its type, default value, and description is available on the [official SQLPage configuration reference](https://sql-page.com/configuration.sql).
The checked-in [JSON Schema](./sqlpage/sqlpage.schema.json) is the source of truth for that reference and for SQLPage's in-memory configuration structure.

Start from the repository's [example configuration](./sqlpage/sqlpage.json). Keep its `$schema` property to get validation and completion in compatible editors.

## Environment variables

Every configuration option can also be supplied as an uppercase environment variable. For example, `database_url` becomes `DATABASE_URL`. Variables prefixed with `SQLPAGE_`, such as `SQLPAGE_DATABASE_URL`, are also accepted.

Values are applied in this order, from lowest to highest priority:

1. built-in defaults;
2. `sqlpage/sqlpage.json`;
3. unprefixed environment variables;
4. `SQLPAGE_`-prefixed environment variables;
5. command-line arguments, where available.

A local `.env` file can be used to define environment variables during development. Do not commit secrets such as database passwords or OIDC client secrets.

## Database connection strings

Set `database_url` (or `DATABASE_URL`) to a URL supported by your database driver, for example:

- SQLite: `sqlite://sqlpage.db?mode=rwc`
- PostgreSQL: `postgres://user:password@localhost/database`
- MySQL: `mysql://user:password@localhost/database`
- Microsoft SQL Server: `mssql://user:password@localhost/database`
- ODBC: an ODBC connection string such as `DSN=DuckDB`

Percent-encode special characters in URL usernames and passwords. You can keep the password separate from the URL with `database_password` or `DATABASE_PASSWORD`.
