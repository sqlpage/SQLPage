SELECT 'shell' as component, 'My public app' as title;

set email = sqlpage.user_info('email');

-- For anonymous users
SELECT 'hero' as component,
    '/protected' as link,
    'Log in' as link_text,
    'Welcome' as title,
    'You are currently browsing as a guest. Log in to access the protected page.' as description,
    '/protected/public/hello.jpeg' as image
WHERE $email IS NULL;

-- For logged-in users
SELECT 'text' as component,
    'Welcome back, ' || sqlpage.user_info('name') || '!' as title,
    'You are logged in as ' || sqlpage.user_info('email') || '. You can now access the [protected page](/protected) or [log out](/logout).' as contents_md
WHERE $email IS NOT NULL;
