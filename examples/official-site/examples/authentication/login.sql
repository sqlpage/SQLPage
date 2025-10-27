select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 
    'login'                 as component,
    'create_session_token.sql'             as action,
    '/assets/icon.webp'   as image,
    'Demo Login Form' as title,
    'Username'              as username,
    'Password'              as password,
    case when $failed is not null then 'Invalid username or password. In this demo, you can log in with admin / admin.' end as error_message,
    'In this demo, the username is "admin" and the password is "admin".' as footer_md,
    'Log in'               as validate;

select 'text' as component, '

# Authentication

This is a simple example of an authentication form.
It uses 
 - the [`login`](/documentation.sql?component=login#component) component to create a login form
 - the [`authentication`](/documentation.sql?component=authentication#component) component to check the user password
 - the [`cookie`](/documentation.sql?component=cookie#component) component to store a unique session token in the user browser
 - the [`redirect`](/documentation.sql?component=redirect#component) component to redirect the user to the login page if they are not logged in 

## Example credentials

 - Username: `admin`, Password: `admin`
 - Username: `user`, Password: `user`
' as contents_md;