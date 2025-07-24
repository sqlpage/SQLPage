SELECT 'shell' as component, 'My public app' as title;

-- For anonymous users
SELECT 'text' as component,
    'You are not logged in.' as title,
    'This is a public page. Click the button below to log in and access the protected page.' as contents_md
    WHERE sqlpage.user_info('email') IS NULL;

SELECT 'button' as component WHERE sqlpage.user_info('email') IS NULL;
SELECT 'Login' as title, 'protected.sql' as link, 'login' as icon WHERE sqlpage.user_info('email') IS NULL;

-- For logged-in users
SELECT 'text' as component,
    'Welcome back, ' || sqlpage.user_info('name') || '!' as title,
    'You are logged in as ' || sqlpage.user_info('email') || '. You can now access the [protected page](protected.sql) or [log out](logout.sql).' as contents_md
    WHERE sqlpage.user_info('email') IS NOT NULL;