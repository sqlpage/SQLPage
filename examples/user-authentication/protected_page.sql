SET username = (SELECT username FROM login_session WHERE id = sqlpage.cookie('session'));

SELECT 'redirect' AS component,
        'signin.sql?error' AS link
WHERE $username IS NULL;

SELECT 'shell' AS component, 'Protected page' AS title, 'lock' AS icon, '/' AS link, 'logout' AS menu_item;

SELECT 'text' AS component,
        'Welcome, ' || $username || ' !' AS title,
        'This content is [top secret](https://youtu.be/dQw4w9WgXcQ).
        You cannot view it if you are not connected.' AS contents_md;