select
    'redirect' as component,
    sqlpage.environment_variable('CAS_ROOT_URL') 
    || '/login?service=' || sqlpage.protocol() || '://' || sqlpage.header('host') || '/cas/redirect_handler.sql'
    as link;