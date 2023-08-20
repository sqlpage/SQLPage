# Configuring SQLPage

SQLPage can be configured through either [environment variables](https://en.wikipedia.org/wiki/Environment_variable)
on a [JSON](https://en.wikipedia.org/wiki/JSON) file placed in `sqlpage/sqlpage.json`.

| variable                                   | default                      | description                                                              |
| ------------------------------------------ | ---------------------------- | ------------------------------------------------------------------------ |
| `listen_on`                                | 0.0.0.0:8080                 | Interface and port on which the web server should listen                 |
| `database_url`                             | sqlite://sqlpage.db?mode=rwc | Database connection URL                                                  |
| `port`                                     | 8080                         | Like listen_on, but specifies only the port.                             |
| `max_database_pool_connections`            | depends on the database      | How many simultaneous database connections to open at most               |
| `database_connection_idle_timeout_seconds` | depends on the database      | Automatically close database connections after this period of inactivity |
| `database_connection_max_lifetime_seconds` | depends on the database      | Always close database connections after this amount of time              |
| `database_connection_retries`              | 6                            | Database connection attempts before giving up. Retries will happen every 5 seconds.           |
| `database_connection_acquire_timeout_seconds` | 10                        | How long to wait when acquiring a database connection from the pool before giving up and returning an error.           |
| `sqlite_extensions`                        |                              | An array of SQLite extensions to load, such as `mod_spatialite`          |

You can find an example configuration file in [`sqlpage/sqlpage.json`](./sqlpage/sqlpage.json).

## Environment variables

All the parameters above can be set through environment variables.

The name of the environment variable is the same as the name of the configuration variable,
but in uppercase.

The environment variable name can optionally be prefixed with `SQLPAGE_`.

### Example

```bash
DATABASE_URL="sqlite:///path/to/my_database.db?mode=rwc"
SQLITE_EXTENSIONS="mod_spatialite crypto define regexp"
```
