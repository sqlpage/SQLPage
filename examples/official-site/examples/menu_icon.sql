SELECT 
    'shell'             AS component,
    'SQLPage'           AS title,
    'database'          AS icon,
    '/'                 AS link,
    TRUE                AS fixed_top_menu,
    '{"title":"About","icon": "settings","submenu":[{"link":"/safety.sql","title":"Security","icon": "logout"},{"link":"/performance.sql","title":"Performance"}]}' AS menu_item,
    '{"title":"Examples","image": "https://upload.wikimedia.org/wikipedia/en/6/6b/Terrestrial_globe.svg","submenu":[{"link":"/examples/tabs.sql","title":"Tabs","image": "https://upload.wikimedia.org/wikipedia/en/6/6b/Terrestrial_globe.svg"},{"link":"/examples/layouts.sql","title":"Layouts"}]}' AS menu_item,
    '{"title":"Examples","size":"sm","image": "https://upload.wikimedia.org/wikipedia/en/6/6b/Terrestrial_globe.svg","submenu":[{"link":"/examples/tabs.sql","title":"Tabs","image": "https://upload.wikimedia.org/wikipedia/en/6/6b/Terrestrial_globe.svg"},{"link":"/examples/layouts.sql","title":"Layouts"}]}' AS menu_item,
    'Official [SQLPage](https://sql.ophir.dev) documentation' as footer;
