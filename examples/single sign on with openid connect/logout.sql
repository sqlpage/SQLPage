-- remove the session cookie
select 'cookie' as component, 'session_id' as name, true as remove;
-- remove the session from the database
delete from user_sessions where session_id = sqlpage.cookie('session_id')
returning 'redirect' as component, -- redirect the user to the oidc provider to logout
    sqlpage.environment_variable('OIDC_END_SESSION_ENDPOINT')
        || '?post_logout_redirect_uri=' || sqlpage.protocol() || '://' || sqlpage.header('host') || '/'
        || '&client_id=' || sqlpage.environment_variable('OIDC_CLIENT_ID')
        || '&id_token_hint=' || oidc_token
    as link;