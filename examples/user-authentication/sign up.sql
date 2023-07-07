SELECT 'form' AS component,
    'Create a new user account' AS title,
    'Sign up' AS validate,
    'create_user.sql' AS action;

SELECT 'username' AS name;
SELECT 'password' AS name, 'password' AS type;
SELECT 'terms' AS name, 'I accept the terms and conditions' AS label, TRUE AS required, FALSE AS value, 'checkbox' AS type;