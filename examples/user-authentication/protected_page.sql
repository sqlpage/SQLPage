
SELECT 'text' AS component,
        'This content is [top secret](https://youtu.be/dQw4w9WgXcQ). You cannot view it if you are not connected.' AS contents_md;

SELECT EXISTS(SELECT 1 FROM login_session WHERE id=sqlpage.cookie('session')) AS contents;
SELECT 'debug' AS component;
SELECT * FROM login_session;
SELECT sqlpage.cookie('session');