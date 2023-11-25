-- redirect to the login page if the password is not correct
SELECT 'authentication' AS component,
    'login.sql?error' AS link,
    (select password_hash from user where username = :Username) AS password_hash,
    :Password AS password;

-- code after this line will only be executed if the user is authenticated
-- (i.e. if the password that they sent matches the password hash that we have stored for them)

insert into session (id, username)
values (sqlpage.random_string(32), :Username)
returning 
    'cookie' AS component,
    'session_token' AS name,
    id AS value;

-- The user browser will now have a cookie named `session_token` that we can check later
-- to see if the user is logged in.
select 'redirect' as component, '/' as link;
