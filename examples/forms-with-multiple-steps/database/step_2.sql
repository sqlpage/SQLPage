update partially_filled_users set name = :name
where :name is not null and id = $id;

select 'form' as component, 'step_3.sql?id=' || $id as action;

select 'email' as name, 'email' as type, true as required, email as value,
    'you@example.com' as placeholder,
    'Hey ' || name || '! what is your email?' as description
from partially_filled_users where id = $id;