INSERT INTO login_session (username)
SELECT username
FROM user_info
WHERE username = :username
    AND password_hash = crypt(:password, password_hash)
RETURNING 'cookie' AS component,
    'session' AS name,
    id AS value;
SELECT 'http_header' AS component,
    'login_check.sql' AS location;