-- Sets the username cookie to the value of the username parameter
SELECT 'cookie' as component,
    'username' as name,
    TRUE as remove;

SELECT 'http_header' as component, 'index.sql' as Location;