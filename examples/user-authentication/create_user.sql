WITH inserted_user AS (
    INSERT INTO user_info (username, password_hash)
    VALUES (:username, sqlpage.hash_password(:password))
    ON CONFLICT (username) DO NOTHING
    RETURNING username
)
SELECT 'hero' AS component,
    'Welcome' AS title,
    'Welcome, ' || username || '! Your user account was successfully created. You can now log in.' AS description,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/e/e1/Community_wp20.png/974px-Community_wp20.png' AS image,
    'signin.sql' AS link,
    'Log in' AS link_text
FROM inserted_user
UNION ALL
SELECT 'hero' AS component,
    'Sorry' AS title,
    'Sorry, this user name is already taken.' AS description_md,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/f/f0/Sad_face_of_a_Wayuu_Woman.jpg/640px-Sad_face_of_a_Wayuu_Woman.jpg' AS image,
    'signup.sql' AS link,
    'Try again' AS link_text
WHERE NOT EXISTS (SELECT 1 FROM inserted_user);