-- Secure OIDC logout with CSRF protection
-- This redirects to /sqlpage/oidc_logout which:
-- 1. Verifies the CSRF token
-- 2. Removes the auth cookies
-- 3. Redirects to the OIDC provider's logout endpoint
-- 4. Finally redirects back to the homepage

select
    'redirect' as component,
    sqlpage.oidc_logout_url('/') as link;
