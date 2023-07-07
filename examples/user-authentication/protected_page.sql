SELECT raise_error('Invalid credentials, please log in') WHERE NOT is_valid_session(sqlpage.cookie('session'));

SELECT 'shell' AS component, 'Protected page' AS title, 'lock' AS icon, '/' AS link, 'logout' AS menu_item;

SELECT 'text' AS component,
        'This content is [top secret](https://youtu.be/dQw4w9WgXcQ). You cannot view it if you are not connected.' AS contents_md;