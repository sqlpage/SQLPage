-- Remove the username cookie
SELECT 'cookie' as component,
    'username' as name,
    TRUE as remove;

SELECT 'http_header' as component, 'index.sql' as Location;