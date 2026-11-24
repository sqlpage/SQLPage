## Including another SQL file

To avoid repeating the same code on multiple pages, you can include another SQL file using the `dynamic` component
together with the [`sqlpage.run_sql`](/functions.sql?function=run_sql) function.

For instance, the following will include the file `shell.sql` at the top of the page,
and pass it a `$title` variable to display the page title.

```sql
SELECT 'dynamic' AS component,
       sqlpage.run_sql('shell.sql', json_object('title', 'SQLPage documentation')) AS properties;
```

And `shell.sql` could contain the following:

```sql
SELECT 'shell'     AS component,
    COALESCE($title, 'Default title') AS title,
    '/my_icon.png' AS icon,
    'products'     AS menu_item,
    'about'        AS menu_item;
```
