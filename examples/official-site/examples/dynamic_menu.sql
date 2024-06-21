SET $dummy = ifnull(:menu, abs(random()) % 5);

SELECT 
    'shell'             AS component,
    'SQLPage'           AS title,
    'database'          AS icon,
    '/'                 AS link,
    iif($dummy = 1, NULL, '{"title":"About","submenu":[{"link":"/safety.sql","title":"Security"},{"link":"/performance.sql","title":"Performance"}]}') AS menu_item,
    iif($dummy = 2, '{}', '{"title":"Examples","submenu":[{"link":"/examples/tabs.sql","title":"Tabs"},{"link":"/examples/layouts.sql","title":"Layouts"}]}') AS menu_item,
    iif($dummy = 3, NULL, '{"title":"Community","submenu":[{"link":"blog.sql","title":"Blog"},{"link":"//github.com/lovasoa/sqlpage/issues","title":"Report a bug"}]}') AS menu_item,
    iif($dummy = 4, '{}', '{"title":"Documentation","submenu":[{"link":"/your-first-sql-website","title":"Getting started"},{"link":"/components.sql","title":"All Components"}]}') AS menu_item,
    'Official [SQLPage](https://sql.ophir.dev) documentation' as footer;


SELECT 
    'form'              AS component,
    'Hide Menu'         AS validate,
    sqlpage.path()      AS action;
SELECT 
    'select'            AS type,
    'menu'              AS name,
    'Hide Menu'         AS label,
    2                   AS width,
    CAST($dummy AS INT) AS value,
    '[{"label": "None",          "value": 0},
      {"label": "About",         "value": 1}, 
      {"label": "Examples",      "value": 2}, 
      {"label": "Community",     "value": 3},
      {"label": "Documentation", "value": 4}]' AS options;
