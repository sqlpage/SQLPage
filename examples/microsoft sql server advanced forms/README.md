# Handling json data in Microsoft SQL Server

This demonstrates both how to produce and read json data from a SQL query
in MS SQL Server (or Azure SQL Database), for creating advanced forms.

This lets your user interact with your database with a simple web interface,
even when you have multiple tables, with one-to-many relationships.

![](./screenshots/app.png)

## Documentation

SQLPage requires JSON to create multi-select input (dropdowns where an user can select multiple values).
The result of these multi-selects is a JSON array, which also needs to be read by SQL queries.

This example demonstrates how to consume [JSON](https://en.wikipedia.org/wiki/JSON) data from a SQL Server database,
using the [`OPENJSON`](https://docs.microsoft.com/en-us/sql/t-sql/functions/openjson-transact-sql)
function to parse the JSON data into a table,
and [`FOR JSON PATH`](https://learn.microsoft.com/en-us/sql/relational-databases/json/format-query-results-as-json-with-for-json-sql-server)
to format query results as a JSON array.


This demonstrates an application designed for managing groups and users, allowing the creation of new groups, adding users, and assigning users to one or multiple groups. 

The application has the following sections:

- **Create a New Group**: A form where users can enter the name of a new group.
- **Groups Display**: A list of existing groups.
- **Add a User**: A form where users can enter the name of a new user and select one or multiple groups to assign to this user.
- **Users Display**: A list of existing users and their associated group memberships.

When users submit the form, their selections are packaged up and sent to the database server. The server receives these selections as a structured JSON array.

The database then takes this list of selections and temporarily converts it into a format it can work with using the `OPENJSON` function, before saving the information permanently in the database tables. This allows the system to process multiple selections at once in an efficient way.
