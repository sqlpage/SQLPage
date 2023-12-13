select 'form' as component;
select 'text' as type, 'Username' as name, username as value
from users where id = $id;

update users set username = :Username
where id = $id and :Username is not null;