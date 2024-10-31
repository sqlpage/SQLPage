select 'form' as component, 'step_2.sql?id=' || $id as action;

select 'name' as name, true as required, name as value
from partially_filled_users where id = $id;