-- remove the session cookie
select 'cookie' as component, 'session_id' as name, true as remove;
-- remove the session from the database
delete from user_sessions where session_id = sqlpage.cookie('session_id');
-- log the user out of the cas server
select
    'redirect' as component,
    sqlpage.environment_variable('CAS_ROOT_URL') 
    || '/logout?service=' || sqlpage.protocol() || '://' || sqlpage.header('host') || '/cas/redirect_handler.sql'
    as link;