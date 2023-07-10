SELECT 'form' AS component,
    'Create a new user account' AS title,
    'Sign up' AS validate,
    'create_user.sql' AS action;

SELECT 'username' AS name;
SELECT 'password' AS name, 'password' AS type, '^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d]{8,}$' AS pattern, 'Password must be at least 8 characters long and contain at least one letter and one number.' AS description;
SELECT 'terms' AS name, 'I accept the terms and conditions' AS label, TRUE AS required, FALSE AS value, 'checkbox' AS type;