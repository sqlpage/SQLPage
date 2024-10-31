update partially_filled_users set email = :email
where :email is not null and id = $id;

select 'form' as component, 'finish.sql?id=' || $id as action;

select 'age' as name, 'number' as type, true as required, age as value,
    'How old are you, ' || name || '?' as description
from partially_filled_users where id = $id;
