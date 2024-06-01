-- Sets the username cookie to the value of the username parameter
SELECT 'cookie' as component,
    'username' as name,
    $username as value
WHERE $username IS NOT NULL;

SELECT 'form' as component;
SELECT 'username' as name,
    'User Name' as label,
    COALESCE($username, sqlpage.cookie('username')) as value,
    'try leaving this page and coming back, the value should be saved in a cookie' as description;

select 'text' as component;
select 'log out' as contents, 'logout.sql' as link;

select 'text' as component;
select 'View the cookie from a subdirectory' as contents, 'subdirectory/read_cookies.sql' as link;