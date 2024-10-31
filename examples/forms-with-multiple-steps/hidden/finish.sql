insert into users (name, email, age) values (:name, :email, :age)
returning
    'alert' as component,
    'Welcome, ' || name || '!' as title,
    'You are user #' || id || '. [Create a new user](step_1.sql)' as description_md;

select 'list' as component, 'Existing users' as title, 'users' as value;
select name as title, email as description from users;