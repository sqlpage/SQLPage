# Configuring SQLPage

SQLPage can be configured through either [environment variables](https://en.wikipedia.org/wiki/Environment_variable)
on a [JSON](https://en.wikipedia.org/wiki/JSON) file placed in `sqlpage/sqlpage.json`.

The name of configuration variables must be written in uppercase when set in environment variables
and in lowercase when set in the configuration file.

| variable                                   | default                      | description                                                              |
|--------------------------------------------|------------------------------|--------------------------------------------------------------------------|
| `listen_on`                                | 0.0.0.0:8080                 | Interface and port on which the web server should listen                 |
| `database_url`                             | sqlite://sqlpage.db?mode=rwc | Database connection URL                                                  |
| `port`                                     | 8080                         | Like listen_on, but specifies only the port.                             |
| `max_database_pool_connections`            | depends on the database      | How many simultaneous database connections to open at most               |
| `database_connection_idle_timeout_seconds` | depends on the database      | Automatically close database connections after this period of inactivity |
| `database_connection_max_lifetime_seconds` | depends on the database      | Always close database connections after this amount of time              |

You can find an example configuration file in [`sqlpage/sqlpage.json`](./sqlpage/sqlpage.json) 