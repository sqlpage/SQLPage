SELECT 'login' AS component,
    'login.sql' AS action,
    'Sign in' AS title,
    'Username' AS username,
    'Password' AS password,
    'user' AS username_icon,
    'lock' AS password_icon,
    case when $error is not null then 'We could not authenticate you. Please log in or [create an account](signup.sql).' end as error_message_md,
    'Sign in' AS validate;