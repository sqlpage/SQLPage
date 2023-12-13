select 'table' as component, 'action' as markdown;
select *,
    format('[Edit](edit.sql?id=%s)', id) as action
from users;