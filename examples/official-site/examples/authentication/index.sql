-- redirect the user to the login page if they are not logged in
-- this query should be present at the top of every page that requires authentication
set user_role = (select role from users natural join user_sessions where session_token = sqlpage.cookie('session_token'));
select 'redirect' as component, 'login.sql' as link where $user_role is null;

select 'dynamic' as component, 
    json_insert(properties, '$[0].menu_item[#]', 'logout') as properties
FROM example WHERE component = 'shell' LIMIT 1;

select 'alert' as component, 'info' as color, CONCAT('You are logged in as ', $user_role) as title;

select 'text' as component, '
# Authentication

Read the [source code](//github.com/lovasoa/SQLpage/blob/main/examples/official-site/examples/authentication/) for this demo.

[Log out](logout.sql)
' as contents_md;