insert into users (
    name, email, age
) values (
    sqlpage.cookie('name'),
    sqlpage.cookie('email'),
    :age -- This is the age that was submitted from the form in step_3.sql
);

-- remove cookies
with t(name) as (values ('name'), ('email'), ('age'))
select 'cookie' as component, name, '/cookies/' as path, true as remove from t;

select
    'alert' as component,
    'Welcome, ' || name || '!' as title,
    'You are user #' || id || '. [Create a new user](step_1.sql)' as description_md
from users where id = last_insert_rowid();

select 'list' as component, 'Existing users' as title, 'users' as value;
select name as title, email as description from users;