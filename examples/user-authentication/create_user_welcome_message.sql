SELECT 'hero' AS component,
    'Welcome' AS title,
    'Welcome, ' || $username || '! Your user account was successfully created. You can now log in.' AS description,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/e/e1/Community_wp20.png/974px-Community_wp20.png' AS image,
    'signin.sql' AS link,
    'Log in' AS link_text
WHERE $error IS NULL;

SELECT 'hero' AS component,
    'Sorry' AS title,
    'Sorry, the user name "' || $username || '" is already taken.' AS description,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/f/f0/Sad_face_of_a_Wayuu_Woman.jpg/640px-Sad_face_of_a_Wayuu_Woman.jpg' AS image,
    'signup.sql' AS link,
    'Try again' AS link_text
WHERE $error IS NOT NULL;