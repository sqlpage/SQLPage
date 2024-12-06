SELECT 'dynamic' AS component, json_object(
    'component', 'shell',
    'title', 'SQLPage documentation',
    'link', '/',
    'menu_item', json_array(
        json_object(
            'link', '/',
            'title', 'Home'
        ),
        json_object(
            'title', 'Community',
            'submenu', json_array(
                json_object(
                    'link', '/blog.sql',
                    'title', 'Blog'
                ),
                json_object(
                    'link', 'https://github.com/sqlpage/SQLPage/issues',
                    'title', 'Issues'
                ),
                json_object(
                    'link', 'https://github.com/sqlpage/SQLPage/discussions',
                    'title', 'Discussions'
                ),
                json_object(
                    'link', 'https://github.com/sqlpage/SQLPage',
                    'title', 'Github'
                )
            )
        )
    ),
    'javascript', json_array(
        'https://cdn.jsdelivr.net/npm/prismjs@1/components/prism-core.min.js',
        'https://cdn.jsdelivr.net/npm/prismjs@1/plugins/autoloader/prism-autoloader.min.js'
    ),
    'css', '/prism-tabler-theme.css'
) AS properties;

select 'text' as component,
'## Dynamic properties

The menu of this page is generated using the `dynamic` component with the following SQL query:

```sql
SELECT ''dynamic'' AS component, json_object(
    ''component'', ''shell'',
    ''title'', ''SQLPage documentation'',
    ''link'', ''/'',
    ''menu_item'', json_array(
        json_object(
            ''link'', ''/'',
            ''title'', ''Home''
        ),
        json_object(
            ''title'', ''Community'',
            ''submenu'', json_array(
                json_object(
                    ''link'', ''/blog.sql'',
                    ''title'', ''Blog''
                ),
                json_object(
                    ''link'', ''https://github.com/sqlpage/SQLPage/issues'',
                    ''title'', ''Issues''
                ),
                json_object(
                    ''link'', ''https://github.com/sqlpage/SQLPage/discussions'',
                    ''title'', ''Discussions''
                ),
                json_object(
                    ''link'', ''https://github.com/sqlpage/SQLPage'',
                    ''title'', ''Github''
                )
            )
        )
    )
) AS properties
```

## Dynamic properties from the database

One could also store the menu items in the database,
using a structure like this:

```sql
CREATE TABLE menu_items (
    id INTEGER PRIMARY KEY,
    title TEXT,
    link TEXT,
    parent_id INTEGER REFERENCES menu_items(id)
);

INSERT INTO menu_items (id, title, link, parent_id) VALUES
    (1, ''Home'', ''/'', NULL),
    (2, ''Community'', NULL, NULL),
    (3, ''Blog'', ''blog.sql'', 2),
    (4, ''Issues'', ''https://github.com/sqlpage/SQLPage/issues'', 2),
    (5, ''Discussions'', ''https://github.com/sqlpage/SQLPage/discussions'', 2),
    (6, ''Github'', ''https://github.com/sqlpage/SQLPage'', 2);
```

Then, one could use the following SQL query to fetch
and generate the menu from the database:

```sql
SELECT ''dynamic'' AS component, json_object(
    ''component'', ''shell'',
    ''title'', ''SQLPage documentation'',
    ''link'', ''/'',
    ''menu_item'', json_group_array(
        json_object(
            ''link'', link,
            ''title'', title,
            ''submenu'', (
                select json_group_array(
                    json_object(
                        ''link'', submenu.link,
                        ''title'', submenu.title
                    )
                ) FROM menu_items AS submenu
                WHERE menu_items.id = submenu.parent_id
                HAVING COUNT(*) > 0
              )
        )
    )
) AS properties
FROM menu_items
WHERE parent_id IS NULL
```
' as contents_md;