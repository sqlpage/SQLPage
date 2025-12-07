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
    'You are logged in as ' || sqlpage.user_info('email') ||
    '. You can now access the [protected page](/protected) or [log out](' ||
    -- Secure OIDC logout with CSRF protection
    -- This redirects to /sqlpage/oidc_logout which:
    -- 1. Verifies the CSRF token
    -- 2. Removes the auth cookies
    -- 3. Redirects to the OIDC provider's logout endpoint
    -- 4. Finally redirects back to the homepage
        sqlpage.oidc_logout_url()
    || ').' as contents_md
WHERE $email IS NOT NULL;
