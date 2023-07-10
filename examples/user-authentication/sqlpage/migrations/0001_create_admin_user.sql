-- Creates an initial user called 'admin'
-- with a password hash that was generated using the 'generate_password_hash.sql' page.
INSERT INTO user_info (username, password_hash)
VALUES ('admin', '$argon2id$v=19$m=19456,t=2,p=1$IiReWDP0ocWvia+fTdozJw$53EozOKX7HkpvOdoWHjsh9yKvRN2TmQm/PjYBeaOqqc');