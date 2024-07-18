# Todo app with SQLPage

This is a simple todo app implemented with SQLPage. It uses a PostgreSQL database to store the todo items.

![Screenshot](screenshot.png)

It is meant as an illustrative example of how to use SQLPage to create a simple CRUD application. See [the SQLite version](../todo%20application/README.md) for a more detailed explanation of the structure of the application.

## Differences from the SQLite version

- URL parameters that contain numeric identifiers are cast to integers using the [`::int`](https://www.postgresql.org/docs/16/sql-expressions.html#SQL-SYNTAX-TYPE-CASTS) operator
- the `printf` function is replaced with the [`format`](https://www.postgresql.org/docs/current/functions-string.html#FUNCTIONS-STRING-FORMAT) function
- primary keys are generated using the [`serial`](https://www.postgresql.org/docs/current/datatype-numeric.html#DATATYPE-SERIAL) type
- dates and times are formatted using the [`to_char`](https://www.postgresql.org/docs/current/functions-formatting.html#FUNCTIONS-FORMATTING-DATETIME-TABLE) function
- the `INSERT OR REPLACE` statement is replaced with the [`ON CONFLICT`](https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT) clause
