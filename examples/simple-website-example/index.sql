-- A form to create a new entry in the database
SELECT 'form' AS component,
    'Add a user' AS title;
SELECT 'Username' as name,
    TRUE as required;
-- Handle the form results when present
INSERT INTO users (username)
SELECT :Username
WHERE :Username IS NOT NULL;
----------------------------------
-- Display the list of users
-- It is important that this query comes after the INSERT query above,
-- so that the updated list is visible immediately
SELECT 'list' AS component,
    'Users' AS title;
SELECT username AS title,
    username || ' is a user on this website.' as description,
    case when is_admin then 'red' end as color,
    'user' as icon,
    'user.sql?id=' || id as link
FROM users;