-- delete expired sessions
delete from user_sessions where created_at < datetime('now', '-1 day');

-- check that the 
SELECT 'authentication' AS component,
    'login.sql?failed' AS link, -- redirect to the login page on error
    (SELECT password_hash FROM users WHERE username = :Username) AS password_hash, -- this is a hash of the password 'admin'
    :Password AS password; -- this is the password that the user sent through our form in 'index.sql'

-- if we haven't been redirected, then the password is correct
-- create a new session
insert into user_sessions (session_token, username) values (sqlpage.random_string(32), :Username)
returning 'cookie' as component, 'session_token' as name, session_token as value;

-- redirect to the authentication example home page
select 'redirect' as component, '/examples/authentication' as link;