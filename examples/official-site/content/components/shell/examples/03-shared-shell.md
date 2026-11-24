### Sharing the shell between multiple pages

It is common to want to share the same shell between multiple pages.

#### Static menu

If your menu is completely static (it does not depend on the database content),
you can use the [`dynamic`](?component=dynamic#component) component together with the
[`sqlpage.read_file_as_text`](functions.sql?function=read_file_as_text#function) function to load the shell from
a json file.

```sql
SELECT 'dynamic' AS component, sqlpage.read_file_as_text('shell.json') AS properties;
```

and in `shell.json`:

```json
{
    "component": "shell",
    "title": "SQL + JSON = <3",
    "link": "/",
    "menu_item": [
        {"link": "index.sql", "title": "Home"},
        {"title": "Community", "submenu": [
            {"link": "/blog.sql", "title": "Blog"},
            {"link": "//github.com/sqlpage/SQLPage", "title": "Github"}
        ]}
    ]
}
```

#### Dynamic menu

If your menu depends on the database content, or on special `sqlpage` functions,
you can use the `dynamic` component,
but this time with the [`sqlpage.run_sql`](functions.sql?function=run_sql#function)
function to generate the menu from the database.

```sql
SELECT 'dynamic' AS component, sqlpage.run_sql('shell.sql') AS properties;
```

and in `shell.sql`:

```sql
SELECT 'shell' AS component, 'run_sql is cool' as title,
    json_group_array(json_object(
        'link', link,
        'title', title
    )) as menu_item
FROM my_menu_items
```

(check your database documentation for the exact syntax of the `json_group_array` function).

Another case when dynamic menus are useful is when you want to show some
menu items only in certain conditions.

For instance, you could show an "Admin panel" menu item only to users with the "admin" role,
a "Profile" menu item only to authenticated users,
and a "Login" menu item only to unauthenticated users:

```sql
set role = (
    SELECT role FROM users
    INNER JOIN sessions ON users.id = sessions.user_id
    WHERE sessions.session_id = sqlpage.cookie('session_id')
); -- Read more about how to handle user sessions in the "authentication" component documentation

SELECT
    'shell' AS component,
    'My authenticated website' AS title,

    -- Add an admin panel link if the user is an admin
    CASE WHEN $role = 'admin' THEN '{"link": "admin.sql", "title": "Admin panel"}' END AS menu_item,

    -- Add a profile page if the user is authenticated
    CASE WHEN $role IS NOT NULL THEN '{"link": "profile.sql", "title": "My profile"}' END AS menu_item,

    -- Add a login link if the user is not authenticated
    CASE WHEN $role IS NULL THEN 'login' END AS menu_item
;
```

More about how to handle user sessions in the [authentication component documentation](?component=authentication#component).

### Menu with icons

The "icon" attribute may be specified for items in the top menu and submenus to display an icon
before the title (or instead). Similarly, the "image" attribute defines a file-based icon. For
image-based icons, the "size" attribute may be specified at the top level of menu_item only to
reduce the size of image-based icons. The following snippet provides an example, which is also
available [here](examples/menu_icon.sql).

```sql
SELECT
    'shell'             AS component,
    'SQLPage'           AS title,
    'database'          AS icon,
    '/'                 AS link,
    TRUE                AS fixed_top_menu,
    '{"title":"About","icon": "settings","submenu":[{"link":"/safety.sql","title":"Security","icon": "logout"},{"link":"/performance.sql","title":"Performance"}]}' AS menu_item,
    '{"title":"Examples","image": "https://upload.wikimedia.org/wikipedia/en/6/6b/Terrestrial_globe.svg","submenu":[{"link":"/examples/tabs/","title":"Tabs","image": "https://upload.wikimedia.org/wikipedia/en/6/6b/Terrestrial_globe.svg"},{"link":"/examples/layouts.sql","title":"Layouts"}]}' AS menu_item,
    '{"title":"Examples","size":"sm","image": "https://upload.wikimedia.org/wikipedia/en/6/6b/Terrestrial_globe.svg","submenu":[{"link":"/examples/tabs/","title":"Tabs","image": "https://upload.wikimedia.org/wikipedia/en/6/6b/Terrestrial_globe.svg"},{"link":"/examples/layouts.sql","title":"Layouts"}]}' AS menu_item,
    'Official [SQLPage](https://sql-page.com) documentation' as footer;
```
