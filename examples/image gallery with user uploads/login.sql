select 'shell' as component, 'My image gallery' as title;

select 'form' as component, 'Login' as title, 'create_session.sql' as action;
select 'text' as type, 'Username' as name, true as required;
select 'password' as type, 'Password' as name, true as required;


select 'alert' as component,
    'danger' as color,
    'You are not logged in' as title,
    'Sorry, we could not log you in. Please try again.' as description
where $error is not null;