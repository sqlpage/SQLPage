select 'button' as component;

set $user_email = (select email from user_sessions where session_id = sqlpage.cookie('session_id'));


select 'Login' as title, '/oidc_login.sql' as link where $user_email is null;
select CONCAT('Currentlty logged in as ',$user_email,'. Log out ?') as title,
    '/oidc_logout.sql' as link where $user_email is not null;