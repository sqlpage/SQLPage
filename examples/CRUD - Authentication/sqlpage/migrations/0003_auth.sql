CREATE TABLE IF NOT EXISTS "accounts" (
    "username" TEXT COLLATE NOCASE PRIMARY KEY,
    "password_hash" TEXT COLLATE BINARY NOT NULL
);

CREATE TABLE IF NOT EXISTS "sessions" (
    "id"         TEXT COLLATE NOCASE PRIMARY KEY,
    "username"   TEXT COLLATE NOCASE NOT NULL
                 REFERENCES "accounts"("username"),
    "created_at" TEXT COLLATE NOCASE NOT NULL DEFAULT CURRENT_TIMESTAMP
);


-- Creates an initial user with the username `admin` and the password `admin` (hashed using sqlpage.hash_password('admin'))

INSERT OR IGNORE INTO "accounts"("username", "password_hash") VALUES
('admin', '$argon2id$v=19$m=19456,t=2,p=1$4lu3hSvaqXK0dMCPZLOIPg$PUFJSB6L3J5eZ33z9WX7y0nOH6KawV2FdW0abMuPE7o');