SELECT 'shell' AS component,
    'User Management App' AS title,
    'user' AS icon,
    '/' AS link,
    CASE COALESCE(sqlpage.cookie('session'), '')
        WHEN '' THEN '["signin", "signup"]'::json
        ELSE '["logout"]'::json
    END AS menu_item;

SELECT 'hero' AS component,
    'SQLPage Authentication Demo' AS title,
    'This application requires signing up to view the protected page.' AS description_md,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/e/e1/Community_wp20.png/974px-Community_wp20.png' AS image,
    'protected_page.sql' AS link,
    'Access protected page' AS link_text;