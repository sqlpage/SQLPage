# SQLPage application example

This is a very simple example of a website that uses the SQLPage web application framework.

This website illustrates how to create a basic Create-Read-Update-Delete (CRUD) application using SQLPage.
It has the following bsic features:

 - Displays a list of user names using the [list component](https://sql.ophir.dev/documentation.sql?component=list#component) (in [`index.sql`](./index.sql#L14-L20))
 - Add a new user name to the list through a [form](https://sql.ophir.dev/documentation.sql?component=form#component) (in [`index.sql`](./index.sql#L1-L9))
 - View a user's personal page by clicking on a name in the list (in [`user.sql`](./user.sql))
 - Delete a user from the list by clicking on the delete button in the user's personal page (in [`delete.sql`](./delete.sql))

## Running the example

To run the example, just launch the sqlpage binary from this directory.

## Files

- [`index.sql`](./index.sql): The main page of the website. Contains a form to add a new user and a list of all users.
- [`user.sql`](./user.sql): The user's personal page. Contains a link to delete the user.
- [`delete.sql`](./delete.sql): The page that deletes the user from the database, and the redirects to the main page.
- [`sqlpage/migrations/001_create_users_table.sql`](./sqlpage/migrations/001_create_users_table.sql): The SQL migration that creates the users table in the database.
- [`sqlpage/sqlpage.json`](./sqlpage/sqlpage.json): The [SQLPage configuration](../../configuration.md) file.
