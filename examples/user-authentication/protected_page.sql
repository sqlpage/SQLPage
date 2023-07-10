SELECT 'redirect' AS component,
        'signin.sql?error' AS link
WHERE logged_in_user(sqlpage.cookie('session')) IS NULL;

SELECT 'shell' AS component, 'Protected page' AS title, 'lock' AS icon, '/' AS link, 'logout' AS menu_item;

SELECT 'text' AS component,
        'Welcome, ' || logged_in_user(sqlpage.cookie('session')) || ' !' AS title,
        'This content is [top secret](https://youtu.be/dQw4w9WgXcQ).
        You cannot view it if you are not connected.' AS contents_md;