update partially_filled_users set age = :age
where :age is not null and id = $id;

insert into users (name, email, age)
select name, email, age from partially_filled_users where id = $id;

delete from partially_filled_users where id = $id;

select
    'alert' as component,
    'Welcome, ' || name || '!' as title,
    'You are user #' || id || '. [Create a new user](index.sql)' as description_md
from users where id = last_insert_rowid();

select 'list' as component, 'Existing users' as title, 'users' as value;
select name as title, email as description from users;