delete from user_sessions
where session_token = sqlpage.cookie('session_token');

select 'redirect' as component, 'login.sql' as link;