# Migrations for Microsoft SQL Server

This folder contains the migrations for the Microsoft SQL Server example.

At the time of writing, SQLPage does not support applying migrations for Microsoft SQL Server
automatically, so we need to apply them manually.

We write the migrations in a folder called `mssql-migrations`, instead of the usual `migrations`
folder, and we use the `sqlcmd` tool to apply them.

See [how it is done in the docker-compose file](../../docker-compose.yml).
