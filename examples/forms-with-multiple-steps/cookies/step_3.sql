select
    'cookie' as component,
    'email' as name,
    :email as value,
    '/cookies/' as path;

select
    'form' as component,
    'finish.sql' as action;

select
    'age' as name,
    'number' as type,
    true as required,
    'How old are you, ' || sqlpage.cookie('name') || '?' as description;