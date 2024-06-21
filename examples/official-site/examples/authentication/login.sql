select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'form' as component, 'Authentication' as title, 'Log in' as validate, 'create_session_token.sql' as action;
select 'Username' as name, 'user' as prefix_icon, 'admin' as placeholder;
select 'Password' as name, 'lock' as prefix_icon, 'admin' as placeholder, 'password' as type;

select 'alert' as component, 'danger' as color, 'Invalid username or password' as title where $failed is not null;

select 'text' as component, '

# Authentication

This is a simple example of an authentication form.
It uses 
 - the [`form`](/documentation.sql?component=form#component) component to create a login form
 - the [`authentication`](/documentation.sql?component=authentication#component) component to check the user password
 - the [`cookie`](/documentation.sql?component=cookie#component) component to store a unique session token in the user browser
 - the [`redirect`](/documentation.sql?component=redirect#component) component to redirect the user to the login page if they are not logged in 

## Example credentials

 - Username: `admin`, Password: `admin`
 - Username: `user`, Password: `user`
' as contents_md;