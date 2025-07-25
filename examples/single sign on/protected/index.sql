set user_email = sqlpage.user_info('email');

select 'shell' as component, 'My secure app' as title,
    'logout' as menu_item;

select 'text' as component,
    'You''re in, '|| sqlpage.user_info('name') || ' !' as title,
    'You are logged in as *`' || $user_email || '`*.

You have access to this protected page.

![open door](/assets/welcome.jpeg)'
    as contents_md;

select 'list' as component;
select key as title, value as description
from json_each(sqlpage.user_info_token());
