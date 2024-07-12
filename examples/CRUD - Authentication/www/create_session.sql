-- Redirect to the login page if the password is not correct

SELECT
    'authentication' AS component,
    'login.sql?' || ifnull('path=' || sqlpage.url_encode($path), '') || '&error=1' AS link,
    :password AS password,
    (SELECT password_hash
     FROM accounts
     WHERE username = :username) AS password_hash;

-- The code after this point is only executed if the user has sent the correct password
-- Generate a random session token and set via the "cookie" component in the RETURNING
-- clause.

INSERT INTO sessions (id, username)
VALUES (sqlpage.random_string(32), :username)
RETURNING 
    'cookie'        AS component,
    'session_token' AS name,
    id              AS value;

-- The user browser will now have a cookie named `session_token` that we can check later
-- to see if the user is logged in.

SELECT
    'redirect' AS component,
    ifnull($path, '/') AS link;
