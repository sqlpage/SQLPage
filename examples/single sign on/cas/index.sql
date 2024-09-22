set user_email = (select email from user_sessions where session_id = sqlpage.cookie('session_id'));

select 'text' as component, 'You are not authenticated. [Log in](login.sql).' as contents_md where $user_email is null;
select 'text' as component, 'Welcome, ' || $user_email || '. You can now [log out](logout.sql).' as contents_md where $user_email is not null;
