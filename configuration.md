# Configuring SQLPage

SQLPage can be configured through either [environment variables](https://en.wikipedia.org/wiki/Environment_variable)
or a [JSON](https://en.wikipedia.org/wiki/JSON) file placed in `sqlpage/sqlpage.json`.

You can find an example configuration file in [`sqlpage/sqlpage.json`](https://github.com/sqlpage/SQLPage/blob/main/sqlpage/sqlpage.json).
See the [configuration reference](https://sql-page.com/configuration) for every option, its accepted values, and its default.

Multiple configuration file formats are supported:
you can use a [`.json5`](https://json5.org/) file, a [`.toml`](https://toml.io/) file, or a [`.yaml`](https://en.wikipedia.org/wiki/YAML#Syntax) file.

## Environment variables

Configuration options can also be set through environment variables.

The name of the environment variable is the same as the name of the configuration variable,
but in uppercase.

For settings read from the configuration file, the environment variable overrides the file, and a variable prefixed with `SQLPAGE_` takes precedence over its unprefixed form.

Additionnally, when troubleshooting, you can set the
[`LOG_LEVEL`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)
environment variable to `sqlpage=debug` to get more detailed logs and see exactly what SQLPage is doing.
Request-completion access logs use the target `sqlpage::access`. Broad filters such as
`sqlpage=info` include them, but target-specific filters such as `sqlpage::webserver::http=info`
must also include `sqlpage::access=info` if you want to keep request logs.

SQLPage also supports [OpenTelemetry](https://opentelemetry.io/) tracing via the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable. See the [SQLPage monitoring example](https://github.com/sqlpage/sqlpage/tree/main/examples/telemetry).

If you have a `.env` file in the current directory or in any of its parent directories, SQLPage will automatically load environment variables from it.

### Database connection strings

The `database_url` parameter sets all the connection parameters for the database, including

 - the database engine type (`sqlite`, `postgres`, `mysql`, `mssql`, or ODBC connection strings)
 - the username and password
 - the host (or ip adress) and port
 - the database name
 - any additional parameters, including
    - `mode=rwc` for SQLite to allow read-write connections
    - `sslmode=require` (or `disable`, `allow`, `verify-ca`, `verify-full`)
     for PostgreSQL to enable or disable SSL
    - `sslrootcert=/path/to/ca.pem` for PostgreSQL to specify the path to the CA certificate file
    - `sslcert=/path/to/cert.pem` to specify the path to the TLS client certificate file and `sslkey=/path/to/key.pem` to specify the path to the TLS client key file for PostgreSQL and MySQL.
    - `application_name=my_application` for PostgreSQL to set the application name, which can be useful for monitoring and logging on the database server side.
    - `collation=utf8mb4_unicode_ci` for MySQL to set the collation of the connection

All the parameters need to be properly [percent-encoded](https://developer.mozilla.org/en-US/docs/Glossary/percent-encoding) if they contain special characters like `@` (`%40`), `:` (`%3A`), `/` (`%2F`), `?` (`%3F`), `#` (`%23`).

A full connection string for a PostgreSQL database might look like this:

```
postgres://my_user:p%40ss@localhost:5432/my_database?sslmode=verify-ca&sslrootcert=/path/to/ca.pem&sslcert=/path/to/cert.pem&sslkey=/path/to/key.pem&application_name=my_application
```

#### ODBC Connection Strings

For ODBC-compatible databases (Oracle, Snowflake, BigQuery, IBM DB2, etc.), you can use ODBC connection strings directly:

```bash
# Using a Data Source Name (DSN)
DATABASE_URL="DSN=MyDatabase"

# Using inline connection parameters
DATABASE_URL="Driver={PostgreSQL};Server=localhost;Port=5432;Database=mydb;UID=myuser;PWD=mypassword"

# Oracle example
DATABASE_URL="Driver={Oracle ODBC Driver};Server=localhost:1521/XE;UID=hr;PWD=password"

# Snowflake example
DATABASE_URL="Driver={SnowflakeDSIIDriver};Server=account.snowflakecomputing.com;Database=mydb;UID=user;PWD=password"
```

ODBC drivers must be installed and configured on your system. On Linux, the `unixODBC` driver manager is statically linked into the SQLPage binary, so you usually only need to install and configure the database-specific ODBC driver for your target database (for example Snowflake, Oracle, DuckDB...).

The [DuckDB image variant](https://github.com/sqlpage/SQLPage#install-your-odbc-database-driver) includes the DuckDB ODBC driver and defaults to a DuckDB database file.

If the `database_password` configuration parameter is set, it will override any password specified in the `database_url`.
It does not need to be percent-encoded.
This allows you to keep the password separate from the connection string, which can be useful for security purposes, especially when storing configurations in version control systems.

### OpenID Connect (OIDC) Authentication

OpenID Connect (OIDC) is a secure way to let users log in to your SQLPage application using their existing accounts from popular services. When OIDC is configured, you can control which parts of your application require authentication using the `oidc_protected_paths` option. By default, all pages are protected. You can specify a list of URL prefixes to protect specific areas, allowing you to have a mix of public and private pages.

To set up OIDC, you'll need to:
1. Register your application with an OIDC provider
2. Configure the provider's details in SQLPage

#### Setting Your Application's Address

When users log in through an OIDC provider, they need to be sent back to your application afterward. For this to work correctly, you need to tell SQLPage where your application is located online:

- Use the `host` setting to specify your application's web address (for example, "myapp.example.com")
- If you already have the `https_domain` setting set (to fetch https certificates for your site), then you don't need to duplicate it into `host`.

Example configuration:
```json
{
  "oidc_issuer_url": "https://accounts.google.com",
  "oidc_client_id": "your-client-id",
  "oidc_client_secret": "your-client-secret",
  "host": "myapp.example.com"
}
```

#### Cloud Identity Providers

- **Google**
  - Documentation: https://developers.google.com/identity/openid-connect/openid-connect
  - Set *oidc_issuer_url* to `https://accounts.google.com`

- **Microsoft Entra ID** (formerly Azure AD)
  - Documentation: https://learn.microsoft.com/en-us/entra/identity-platform/quickstart-register-app
  - Set *oidc_issuer_url* to `https://login.microsoftonline.com/{tenant}/v2.0`
    - ([Find your tenant name](https://learn.microsoft.com/en-us/entra/identity-platform/v2-protocols-oidc#find-your-apps-openid-configuration-document-uri))

- **GitHub**
  - Issuer URL: `https://github.com`
  - Documentation: https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps

#### Self-Hosted Solutions

- **Keycloak**
  - Issuer URL: `https://your-keycloak-server/auth/realms/your-realm`
  - [Setup Guide](https://www.keycloak.org/getting-started/getting-started-docker)

- **Authentik**
  - Issuer URL: `https://your-authentik-server/application/o/your-application`
  - [Setup Guide](https://goauthentik.io/docs/providers/oauth2)

After registering your application with the provider, you'll receive a client ID and client secret. These are used to configure SQLPage to work with your chosen provider.

Note: OIDC is optional. If you don't configure it, your SQLPage application will be accessible without authentication.

### Example `.env` file

```bash
DATABASE_URL="postgres://my_user@localhost:5432/my_database?sslmode=verify-ca&sslrootcert=/path/to/ca.pem"
DATABASE_PASSWORD="my_secure_password"
SQLITE_EXTENSIONS="mod_spatialite crypto define regexp"
```

## Custom components

SQLPage allows you to create custom components in addition to or instead of the default ones.
To create a custom component, create a [`.handlebars`](https://handlebarsjs.com/guide/expressions.html)
file in the `sqlpage/templates` directory of your SQLPage installation.

For instance, if you want to create a custom `my_component` component, that displays the value of the `my_column` column, create a `sqlpage/templates/my_component.handlebars` file with the following content:

```handlebars
<ul>
    {{#each_row}}
        <li>Value of my column: {{my_column}}</li>
    {{/each_row}}
</ul>
```

[See the full custom component documentation](https://sql-page.com/custom_components.sql).

## Directories

SQLPage needs two important directories to work: the configuration directory, and the web root.

### Configuration directory

The configuration directory is the `./sqlpage/` folder by default.
In the [official docker image](https://hub.docker.com/r/lovasoa/sqlpage), it is located in `/etc/sqlpage/`.
It can be configured using the `--config-dir` command-line argument, or the `SQLPAGE_CONFIGURATION_DIRECTORY` environment variable.

At startup, SQLPage creates the configuration directory when it is absent and rejects the path when it is not a directory.
AWS Lambda release artifacts include an empty `./sqlpage` directory because deployed function packages are read-only.

It can contain

 - the [`sqlpage.json`](#configuring-sqlpage) configuration file,
 - the [`templates`](#custom-components) directory,
 - the [`migrations`](#migrations) directory,
 - the [connection management](#connection-management) sql files.

### Web Root

The web root is where you place your sql files.
By default, it is set to the current working directory, from which the sqlpage binary is launched.
In the [official docker image](https://hub.docker.com/r/lovasoa/sqlpage), the web root is set to `/var/www`.
It can be configured using the `--web-root` command-line argument, or the `SQLPAGE_WEB_ROOT` environment variable.

## Connection management

### Connection initialization scripts

SQLPage allows you to run a SQL script when a new database connection is opened,
by simply creating a `sqlpage/on_connect.sql` file.

This can be useful to set up the database connection for your application.
For instance, on postgres, you can use this to [set the `search path` and various other connection options](https://www.postgresql.org/docs/current/sql-set.html).

```sql
SET TIME ZONE 'UTC';
SET search_path = my_schema;
```

On SQLite, you can use this to [`ATTACH`](https://www.sqlite.org/lang_attach.html) additional databases.

```sql
ATTACH DATABASE '/path/to/my_other_database.db' AS my_other_database;
```

(and then, you can use `my_other_database.my_table` in your queries)

You can also use this to create *temporary tables* to store intermediate results that are useful in your SQLPage application, but that you don't want to store permanently in the database.

```sql
CREATE TEMPORARY TABLE my_temporary_table(
    my_temp_column TEXT
);
```

### Connection cleanup scripts: `on_reset.sql`

SQLPage allows you to run a SQL script after a request has been processed,
by simply creating a `sqlpage/on_reset.sql` file.

This can be useful to clean up temporary tables,
rollback transactions that were left open,
or other resources that were created during the request.

You can also use this script to close database connections that are
in an undesirable state, such as being in a transaction that was left open.
To close a connection, write a select statement that returns a single row
with a single boolean column named `is_healthy`, and set it to false.

#### Rollback transactions

You can automatically rollback any open transactions
when a connection is returned to the pool,
so that a new request is never executed in the context of an open transaction from a previous request.

For this to work, you need to create a `sqlpage/on_reset.sql` containing the following line:

```sql
ROLLBACK;
```

#### Cleaning up all connection state

Some databases allow you to clean up all the state associatPed with a connection.

##### PostgreSQL

By creating a `sqlpage/on_reset.sql` file containing a [`DISCARD ALL`](https://www.postgresql.org/docs/current/sql-discard.html) statement.

```sql
DISCARD ALL;
```

##### SQL Server

By creating a `sqlpage/on_reset.sql` file containing a call to the [`sp_reset_connection`](https://learn.microsoft.com/en-us/sql/relational-databases/system-stored-procedures/system-stored-procedures-transact-sql?view=sql-server-ver16#api-system-stored-procedures) stored procedure.

```sql
EXEC sp_reset_connection;
```

## Migrations

SQLPage allows you to run SQL scripts when the database schema changes, by creating a `sqlpage/migrations` directory.
We have a guide on [how to create migrations](https://sql-page.com/your-first-sql-website/migrations.sql).

## Custom URL routes

By default, SQLPage encourages a simple mapping between the URL and the SQL file that is executed.
You can also create custom URL routes by creating [`404.sql` files](https://sql-page.com/your-first-sql-website/custom_urls.sql).
If you need advanced routing, you can also [add a reverse proxy in front of SQLPage](https://sql-page.com/your-first-sql-website/nginx.sql).
