-- remove the session cookie
select
    'cookie' as component,
    'sqlpage_auth' as name,
    true as remove;

select
    'redirect' as component,
    'http://localhost:8181/realms/sqlpage_demo/protocol/openid-connect/logout' as link;