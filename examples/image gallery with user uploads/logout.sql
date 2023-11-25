select
    'cookie' AS component,
    'session_token' AS name,
    true AS remove;

select 'redirect' as component, '/login.sql' as link -- redirect to the login page after the user logs out
