-- Remove the username cookie
SELECT 'cookie' as component,
    'username' as name,
    TRUE as remove;

SELECT 'redirect' as component, 'index.sql' as link;