-- The authentication component will stop the execution of the page and redirect the user to the login page if
-- the password is incorrect or if the user does not exist.
SELECT 'authentication' AS component,
    'signin.sql?error' AS link,
    (SELECT password_hash FROM user_info WHERE username = :username) AS password_hash,
    :password AS password;

-- Generate a random 32 characters session ID, insert it into the database,
-- and save it in a cookie on the user's browser.
INSERT INTO login_session (id, username)
VALUES (sqlpage.random_string(32), :username)
RETURNING 
    'cookie' AS component,
    'session' AS name,
    id AS value,
    FALSE AS secure; -- You can remove this if the site is served over HTTPS.

-- Redirect the user to the protected page.
SELECT 'redirect' AS component, 'protected_page.sql' AS link;
