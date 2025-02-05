select 'json' as component;

select name as value, name as label
from component
where name like '%' || $search || '%';