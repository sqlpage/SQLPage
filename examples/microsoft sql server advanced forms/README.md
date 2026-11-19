# Onboard users and assign groups with SQL Server

Create groups, then use a SQLPage form to onboard users and assign their
memberships. A SQL Server stored procedure saves the user and memberships together.

![](./screenshots/app.png)

## Try it

Run `docker compose up` in this directory and open the [onboarding page](http://localhost:8080/).
The database container runs `sqlcmd` to create the tables and stored procedure.

1. Create two groups. A blank description is saved as SQL `NULL`.
2. Under **Add a user**, enter a name and select both groups. You can also add a user with no groups.
3. Submit the form and check the memberships under **Users**.

## How it works

- SQL Server returns group options with `FOR JSON PATH`. SQLPage uses the JSON to render a multi-select.
- SQLPage binds the submitted name and the array of selected group IDs to `dbo.onboard_user`. The procedure uses `SCOPE_IDENTITY()` for the new user ID and `OPENJSON` to turn the array into membership rows.
- The procedure saves the user and memberships in one transaction. If a selected group has been deleted since the form loaded, the foreign key rejects the selection and SQL Server rolls back the entire transaction.
- `STRING_AGG` displays each user's groups in creation order.

Run the end-to-end test from the repository root.

```sh
scripts/test-examples-hurl.sh 'examples/microsoft sql server advanced forms'
```
