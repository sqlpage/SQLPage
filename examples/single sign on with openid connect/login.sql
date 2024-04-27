set $oauth_state = sqlpage.random_string(32);

SELECT 'cookie' as component, 'oauth_state' as name, $oauth_state as value;

select 'redirect' as component,
    'http://localhost:8181/realms/sqlpage_demo/protocol/openid-connect/auth' -- replace this with the URL of your OpenID Connect provider
        || '?response_type=code'
        || '&client_id=' || sqlpage.url_encode(sqlpage.environment_variable('OIDC_CLIENT_ID'))
        || '&redirect_uri=http://localhost:8080/oidc_redirect_handler.sql' -- replace this with the URL of your application
        || '&state=' || $oauth_state
        || '&scope=openid+profile+email'
        || '&nonce=' || sqlpage.random_string(32)
    as link;