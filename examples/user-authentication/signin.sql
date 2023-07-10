SELECT 'form' AS component,
    'Sign in' AS title,
    'Sign in' AS validate,
    'login.sql' AS action;

SELECT 'username' AS name;
SELECT 'password' AS name, 'password' AS type;

SELECT 'alert' as component,
    'Sorry' as title,
    'We could not authenticate you. Please log in or [create an account](signup.sql).' as description_md,
    'alert-circle' as icon,
    'red' as color
WHERE $error IS NOT NULL;