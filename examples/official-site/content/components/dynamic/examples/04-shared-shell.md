## Dynamic shell

On databases without a native JSON type (such as the default SQLite database),
you can use the `dynamic` component to generate
json data to pass to components that expect it.

This example generates a menu similar to the [shell example](?component=shell#component), but without using a native JSON type.

```sql
SELECT 'dynamic' AS component, '
{
    "component": "shell",
    "title": "SQLPage documentation",
    "link": "/",
    "menu_item": [
        {"link": "index.sql", "title": "Home"},
        {"title": "Community", "submenu": [
            {"link": "blog.sql", "title": "Blog"},
            {"link": "https//github.com/sqlpage/SQLPage/issues", "title": "Issues"},
            {"link": "https//github.com/sqlpage/SQLPage/discussions", "title": "Discussions"},
            {"link": "https//github.com/sqlpage/SQLPage", "title": "Github"}
        ]}
    ]
}
' AS properties
```

[View the result of this query, as well as an example of how to generate a dynamic menu
based on the database contents](./examples/dynamic_shell.sql).
