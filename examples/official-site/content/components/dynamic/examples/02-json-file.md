## Static component data stored in `.json` files

You can also store the data for a component in a `.json` file, and load it using the `dynamic` component.

This is particularly useful to create a single [shell](?component=shell#component) defining the site's overall appearance and menus,
and displaying it on all pages without duplicating its code.

The following will load the data for a `shell` component from a file named `shell.json`,
using the [`sqlpage.read_file_as_text`](/functions.sql?function=read_file_as_text) function.

```sql
SELECT 'dynamic' AS component, sqlpage.read_file_as_text('shell.json') AS properties;
```

and `shell.json` would be placed at the website's root and contain the following:

```json
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
```
