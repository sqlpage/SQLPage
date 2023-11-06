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
| `web_root`                                | `.`                      | The root directory of the web server, where the `index.sql` file is located.          |
| `allow_exec` | false | Allow usage of the `sqlpage.exec` function. Do this only if all users with write access to sqlpage query files and to the optional `sqlpage_files` table on the database are trusted. |

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