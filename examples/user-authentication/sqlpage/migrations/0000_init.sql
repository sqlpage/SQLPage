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