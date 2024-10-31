select
    'cookie' as component,
    'name' as name,
    :name as value,
    '/cookies/' as path; -- Only send the cookie for pages in the /cookies/ directory

select
    'form' as component,
    'step_3.sql' as action;

select
    'email' as name,
    'email' as type,
    true as required,
    sqlpage.cookie ('email') as value,
    'you@example.com' as placeholder,
    'Hey ' || coalesce(:name, sqlpage.cookie('name')) || '! what is your email?' as description;
