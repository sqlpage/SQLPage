SELECT 'shell' AS component, 'User Management App' AS title, 'user' AS icon, '/' AS link;

SELECT 'form' AS component,
    'Edit user' AS title,
    CASE WHEN $id IS NULL THEN 'insert_user.sql'
                          ELSE 'update_user.sql?id=' || $id
    END AS action;

SELECT 'First name' AS name,
    TRUE AS required,
    (SELECT first_name FROM user WHERE id=$id) AS value;

SELECT 'Last name' AS name,
    TRUE AS required,
    (SELECT last_name FROM user WHERE id=$id) AS value;

SELECT 'Email' AS name,
    'email' AS type,
    (SELECT email FROM user WHERE id=$id) AS value;

SELECT 'list' AS component, 'Addresses' AS title WHERE $id IS NOT NULL;
SELECT street || ', ' || city || ', ' || country AS title FROM address WHERE user_id=$id;

SELECT 'form' AS component, 'Add address' AS title, 'insert_address.sql?user_id=' || $id AS action WHERE $id IS NOT NULL;
SELECT 'Street' AS name, TRUE AS required WHERE $id IS NOT NULL;
SELECT 'City' AS name, TRUE AS required WHERE $id IS NOT NULL;
SELECT 'Country' AS name, TRUE AS required WHERE $id IS NOT NULL;