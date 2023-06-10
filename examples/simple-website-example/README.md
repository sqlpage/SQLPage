# SQLPage application example

This is a very simple example of a website that uses the SQLPage web application framework.

This website illustrates how to create a very Create-Read-Update-Delete (CRUD) application using SQLPage.
It has the following bsic features:

 - Displays a list of user names using the [list component](https://sql.ophir.dev/documentation.sql?component=list#component) (in [`index.sql`](./index.sql#L14-L20))
 - Add a new user name to the list through a form (in [`index.sql`](./index.sql#L1-L9))
 - View an user's personal page by clicking on a name in the list (in [`user.sql`](./user.sql))
 - Delete an user from the list by clicking on the delete button in the user's personal page (in [`delete.sql`](./delete.sql))