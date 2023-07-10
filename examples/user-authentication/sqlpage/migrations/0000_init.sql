CREATE TABLE user_info (
    username TEXT PRIMARY KEY,
    password_hash TEXT NOT NULL
);

CREATE TABLE login_session (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL REFERENCES user_info(username),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- A small pure utility function to get the current user from a session cookie.
-- In a database that does not support functions, you could inline this query
-- or use a view if you need more information from the user table
CREATE FUNCTION logged_in_user(session_id TEXT) RETURNS TEXT AS $$
    SELECT username FROM login_session WHERE id = session_id;
$$ LANGUAGE SQL STABLE;
