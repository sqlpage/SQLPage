update users
set username = :Username,
    is_admin = :Administrator is not null
where :Username is not null and id = $id;

select 'form' as component;
select 'text' as type, 'Username' as name, username as value from users where id = $id;
select 'checkbox' as type, 'Has administrator privileges' as label, 'Administrator' as name, is_admin as checked from users where id = $id;