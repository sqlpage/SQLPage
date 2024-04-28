set $user_email = (select email from user_sessions where session_id = sqlpage.cookie('session_id'));

select 'shell' as component, 'My secure app' as title,
    (case when $user_email is null then 'login' else 'logout' end) as menu_item;

select 'text' as component, sqlpage.read_file_as_text('assets/homepage.md') as contents_md where $user_email is null;

select 'text' as component,
    'You''re in !' as title,
    'You are now logged in as *`' || $user_email || '`*.
You have access to the [protected page](protected.sql).

![open door](/assets/welcome.jpeg)'
        as contents_md
where $user_email is not null;