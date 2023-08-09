WITH logged_in AS (
    SELECT COALESCE(sqlpage.cookie('session'), '') <> '' AS logged_in
)
SELECT 'shell' AS component,
    'User Management App' AS title,
    'user' AS icon,
    '/' AS link,
    json_agg(menu_items.link) AS menu_item
FROM (
    SELECT 'signin' AS link FROM logged_in WHERE NOT logged_in
    UNION ALL
    SELECT 'signup' FROM logged_in WHERE NOT logged_in
    UNION ALL
    SELECT 'logout' FROM logged_in WHERE logged_in
) AS menu_items;

SELECT 'hero' AS component,
    'SQLPage Authentication Demo' AS title,
    'This application requires signing up to view the protected page.' AS description_md,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/e/e1/Community_wp20.png/974px-Community_wp20.png' AS image,
    'protected_page.sql' AS link,
    'Access protected page' AS link_text;