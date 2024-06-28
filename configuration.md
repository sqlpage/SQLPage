# Configuring SQLPage

SQLPage can be configured through either [environment variables](https://en.wikipedia.org/wiki/Environment_variable)
or a [JSON](https://en.wikipedia.org/wiki/JSON) file placed in `sqlpage/sqlpage.json`.

You can find an example configuration file in [`sqlpage/sqlpage.json`](./sqlpage/sqlpage.json).
Here are the available configuration options and their default values:

| variable                                      | default                                                     | description                                                                                                                                                                                                                                            |
| --------------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `listen_on`                                   | 0.0.0.0:8080                                                | Interface and port on which the web server should listen                                                                                                                                                                                               |
| `database_url`                                | sqlite://sqlpage.db?mode=rwc                                | Database connection URL, in the form `dbname://user:password@host:port/dbname`. Special characters in user and password should be [percent-encoded](https://developer.mozilla.org/en-US/docs/Glossary/percent-encoding). |
| `port`                                        | 8080                                                        | Like listen_on, but specifies only the port.                                                                                                                                                                                                           |
| `unix_socket`                                 |                                                             | Path to a UNIX socket to listen on instead of the TCP port. If specified, SQLPage will accept HTTP connections only on this socket and not on any TCP port. This option is mutually exclusive with `listen_on` and `port`.
| `max_database_pool_connections`               | PostgreSQL: 50<BR>  MySql: 75<BR> SQLite: 16<BR> MSSQL: 100 | How many simultaneous database connections to open at most                                                                                                                                                                                             |
| `database_connection_idle_timeout_seconds`    | SQLite: None<BR> All other: 30 minutes                      | Automatically close database connections after this period of inactivity                                                                                                                                                                               |
| `database_connection_max_lifetime_seconds`    | SQLite: None<BR> All other: 60 minutes                      | Always close database connections after this amount of time                                                                                                                                                                                            |
| `database_connection_retries`                 | 6                                                           | Database connection attempts before giving up. Retries will happen every 5 seconds.                                                                                                                                                                    |
| `database_connection_acquire_timeout_seconds` | 10                                                          | How long to wait when acquiring a database connection from the pool before giving up and returning an error.                                                                                                                                           |
| `sqlite_extensions`                           |                                                             | An array of SQLite extensions to load, such as `mod_spatialite`                                                                                                                                                                                        |
| `web_root`                                    | `.`                                                         | The root directory of the web server, where the `index.sql` file is located.                                                                                                                                                                           |
| `site_prefix`                                 | `/`                                                         | Base path of the site. If you want to host SQLPage at `https://example.com/sqlpage/`, set this to `/sqlpage/`. When using a reverse proxy, this allows hosting SQLPage together with other applications on the same subdomain. |
| `configuration_directory`                     | `./sqlpage/`                                                | The directory where the `sqlpage.json` file is located. This is used to find the path to [`templates/`](https://sql.ophir.dev/custom_components.sql), [`migrations/`](https://sql.ophir.dev/your-first-sql-website/migrations.sql), and `on_connect.sql`. Obviously, this configuration parameter can be set only through environment variables, not through the `sqlpage.json` file itself in order to find the `sqlpage.json` file. Be careful not to use a path that is accessible from the public WEB_ROOT |
| `allow_exec`                                  | false                                                       | Allow usage of the `sqlpage.exec` function. Do this only if all users with write access to sqlpage query files and to the optional `sqlpage_files` table on the database are trusted.                                                                  |
| `max_uploaded_file_size`                      | 5242880                                                     | Maximum size of uploaded files in bytes. Defaults to 5 MiB.                                                                                                                                                                                            |
| `max_pending_rows`                            | 256                                                         | Maximum number of rendered rows that can be queued up in memory when a client is slow to receive them. |
| `compress_responses`                          | true                                                        | When the client supports it, compress the http response body. This can save bandwidth and speed up page loading on slow connections, but can also increase CPU usage and cause rendering delays on pages that take time to render (because streaming responses are buffered for longer than necessary). |
| `https_domain`                                |                                                             | Domain name to request a certificate for. Setting this parameter will automatically make SQLPage listen on port 443 and request an SSL certificate. The server will take a little bit longer to start the first time it has to request a certificate.  |
| `https_certificate_email`                     | contact@<https_domain>                                      | The email address to use when requesting a certificate.                                                                                                                                                                                                |
| `https_certificate_cache_dir`                 | ./sqlpage/https                                             | A writeable directory where to cache the certificates, so that SQLPage can serve https traffic immediately when it restarts.                                                                                                                           |
| `https_acme_directory_url`                    | https://acme-v02.api.letsencrypt.org/directory              | The URL of the ACME directory to use when requesting a certificate.                                                                                                                                                                                    |
| `environment`                                 | development                                                 | The environment in which SQLPage is running. Can be either `development` or `production`. In `production` mode, SQLPage will hide error messages and stack traces from the user, and will cache sql files in memory to avoid reloading them from disk. |

Multiple configuration file formats are supported:
you can use a [`.json5`](https://json5.org/) file, a [`.toml`](https://toml.io/) file, or a [`.yaml`](https://en.wikipedia.org/wiki/YAML#Syntax) file.

## Environment variables

All the parameters above can be set through environment variables.

The name of the environment variable is the same as the name of the configuration variable,
but in uppercase.

The environment variable name can optionally be prefixed with `SQLPAGE_`.

Additionnally, when troubleshooting, you can set the [`RUST_LOG`](https://docs.rs/env_logger/latest/env_logger/#enabling-logging)
environment variable to `sqlpage=debug` to get more detailed logs and see exactly what SQLPage is doing.

If you have a `.env` file in the current directory or in any of its parent directories, SQLPage will automatically load environment variables from it.

### Database connection strings

The `database_url` parameter sets all the connection parameters for the database, including

 - the database type (`sqlite`, `postgres`, `mysql`, `mssql`, etc.)
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

All the parameters need to be properly [percent-encoded](https://developer.mozilla.org/en-US/docs/Glossary/percent-encoding) if they contain special characters.

A full connection string for a PostgreSQL database might look like this:

```
postgres://my_user:p%40ss@localhost:5432/my_database?sslmode=verify-ca&sslrootcert=/path/to/ca.pem&sslcert=/path/to/cert.pem&sslkey=/path/to/key.pem&application_name=my_application
```

### Example `.env` file

```bash
DATABASE_URL="sqlite:///path/to/my_database.db?mode=rwc"
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

[See the full custom component documentation](https://sql.ophir.dev/custom_components.sql).

## Connection initialization scripts

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
