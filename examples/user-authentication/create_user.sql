WITH inserted_user AS (
    INSERT INTO user_info (username, password_hash)
    VALUES (:username, crypt(:password, gen_salt('bf', 10)))
    ON CONFLICT (username) DO NOTHING
    RETURNING username
)
SELECT 'text' AS component,
    COALESCE(
        'Welcome, ' || (SELECT username FROM inserted_user) || '! Your user account was successfully created. You can now [log in](sign%20in.sql).',
        'Sorry, this user name is already taken.'
    ) AS contents_md;