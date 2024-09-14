set oauth_state = sqlpage.random_string(32);

SELECT 'cookie' as component, 'oauth_state' as name, $oauth_state as value;

select 'redirect' as component,
    sqlpage.environment_variable('OIDC_AUTHORIZATION_ENDPOINT') 
        || '?response_type=code'
        || '&client_id=' || sqlpage.url_encode(sqlpage.environment_variable('OIDC_CLIENT_ID'))
        || '&redirect_uri=' || sqlpage.protocol() || '://' || sqlpage.header('host') || '/oidc_redirect_handler.sql'
        || '&state=' || $oauth_state
        || '&scope=openid+profile+email'
        || '&nonce=' || sqlpage.random_string(32)
    as link;