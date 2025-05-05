set user_email = sqlpage.user_info('email');

select 'shell' as component, 'My secure app' as title,
    'logout' as menu_item;

select 'text' as component,
    'You''re in !' as title,
    'You are logged in as *`' || $user_email || '`*.
You have access to the [protected page](protected.sql).

![open door](/assets/welcome.jpeg)'
    as contents_md;

select 'list' as component;
select key as title, value as description
from json_each(sqlpage.id_token());