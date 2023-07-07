CREATE TABLE user_info (
    username TEXT PRIMARY KEY,
    password_hash TEXT NOT NULL
);

-- Activate the pgcrypto extension to be able to hash passwords, and generate session IDs.
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE login_session (
    id TEXT PRIMARY KEY DEFAULT encode(gen_random_bytes(128), 'hex'),
    username TEXT NOT NULL REFERENCES user_info(username),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);


-- Returns true if the session is valid, false otherwise
CREATE FUNCTION is_valid_session(user_session text) RETURNS boolean AS $$
BEGIN
    RETURN EXISTS(SELECT 1 FROM login_session WHERE id=user_session);
END;
$$ LANGUAGE plpgsql;

-- Takes a session id, does nothing if it is valid, throws an error otherwise.
CREATE FUNCTION raise_error(error_message_text text) RETURNS void AS $$
BEGIN
    RAISE EXCEPTION '%', error_message_text;
END;
$$ LANGUAGE plpgsql;