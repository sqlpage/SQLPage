select 'shell' as component, 'My public app' as title;

select 'text' as component,
    'This is a public page. You can see it without being logged in.' as title,
    'Click the button below to log in and access the protected page.' as contents_md;

select 'button' as component;
select 'Login' as title, 'protected.sql' as link, 'login' as icon;