-- remove the session cookie
select
    'cookie' as component,
    'sqlpage_auth' as name,
    true as remove;

select
    'redirect' as component,
    sqlpage.link('http://localhost:8181/realms/sqlpage_demo/protocol/openid-connect/logout', json_object(
    'post_logout_redirect_uri', 'http://localhost:8080/',
    'id_token_hint', sqlpage.cookie('sqlpage_auth')
    )) as link;
