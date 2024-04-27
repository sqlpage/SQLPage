-- remove the session cookie
select 'cookie' as component, 'session_id' as name, true as remove;
-- remove the session from the database
delete from user_sessions where session_id = sqlpage.cookie('session_id')
returning 'redirect' as component, -- redirect the user to the oidc provider to logout
    'http://localhost:8181/realms/sqlpage_demo/protocol/openid-connect/logout' -- replace this with the logout URL of your OpenID Connect provider
        || '?post_logout_redirect_uri=' || sqlpage.url_encode('http://localhost:8080/') -- replace this with the URL of your application
        || '&client_id=' || sqlpage.environment_variable('OIDC_CLIENT_ID')
        || '&id_token_hint=' || oidc_token
    as link;