# SQLPage migrations

SQLPage migrations are SQL scripts that you can use to create or update the database schema.
They are entirely optional: you can use SQLPage without them, and manage the database schema yourself with other tools.

If you are new to SQL migrations, please read our [**introduction to database migrations**](https://sql-page.com/your-first-sql-website/migrations.sql).

## Creating a migration

To create a migration, create a file in the `sqlpage/migrations` directory with the following name:

```
<version>_<name>.sql
```

Where `<version>` is a number that represents the version of the migration, and `<name>` is a name for the migration.
For example, `001_initial.sql` or `002_add_users.sql`.

When you need to update the database schema, always create a **new** migration file with a new version number
that is greater than the previous one.
Use commands like `ALTER TABLE` to update the schema declaratively instead of modifying the existing `CREATE TABLE`
statements.

If you try to edit an existing migration, SQLPage will not run it again, it will detect that the migration has already executed. Also, if the migration is different than the one that was executed, SQLPage will throw an error as the database structure must match.

## Creating migrations on the command line

You can create a migration directly with sqlpage by running the command "sqlpage create-migration [migration_name]"

For example if you run 'sqlpage create-migration "Example Migration 1"' on the command line, you will find a new file under "sqlpage/migrations" folder called "[timestamp]_example_migration_1.sql" where timestamp is the current time when you ran the command.

## Running migrations

Migrations that need to be applied are run automatically when SQLPage starts.
You need to restart SQLPage each time you create a new migration.

## How does it work?

SQLPage keeps track of the migrations that have been applied in a table called `_sqlx_migrations`.
This table is created automatically when SQLPage starts for the first time, if you create migration files.
If you don't create any migration files, SQLPage will never touch the database schema on its own.

When SQLPage starts, it checks the `_sqlx_migrations` table to see which migrations have been applied.
It checks the `sqlpage/migrations` directory to see which migrations are available.
If the checksum of a migration file is different from the checksum of the migration that has been applied,
SQLPage will return an error and refuse to start.
If you end up in this situation, you can remove the `_sqlx_migrations` table: all your old migrations will be reapplied, and SQLPage will start again.
