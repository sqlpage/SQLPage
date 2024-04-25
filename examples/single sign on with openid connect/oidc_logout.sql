-- remove the session cookie
select 'cookie' as component, 'session_id' as name, true as remove;
-- remove the session from the database
delete from user_sessions
    where session_id = sqlpage.cookie('session_id');
-- redirect the user to the oidc provider to logout
select 'redirect' as component,
    'http://localhost:8181/realms/sqlpage_demo/protocol/openid-connect/logout' -- replace this with the logout URL of your OpenID Connect provider
        || '?redirect_url=http://localhost:8080/' -- replace this with the URL of your application
    as link;