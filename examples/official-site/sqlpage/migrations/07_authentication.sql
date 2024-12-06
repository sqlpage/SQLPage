-- Insert the http_header component into the component table
INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'authentication',
        'An advanced component that can be used to create pages with password-restricted access.
        When used, this component has to be at the top of your page, because once the page has begun being sent to the browser, it is too late to restrict access to it.
        The authentication component checks if the user has sent the correct password, and if not, redirects them to the URL specified in the link parameter.
        If you don''t want to re-check the password on every page (which is an expensive operation),
        you can check the password only once and store a session token in your database. 
        You can use the cookie component to set the session token cookie in the client browser,
        and then check whether the token matches what you stored in subsequent pages.',
        'lock',
        '0.7.2'
    );
-- Insert the parameters for the http_header component into the parameter table
INSERT INTO parameter (
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
VALUES (
        'authentication',
        'link',
        'The URL to redirect the user to if they are not logged in. If this parameter is not specified, the user will stay on the current page, but be asked to log in using a popup in their browser (HTTP basic authentication).',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'authentication',
        'password',
        'The password that was sent by the user. You can set this to :password if you have a login form leading to your page.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'authentication',
        'password_hash',
        'The hash of the password that you stored for the user that is currently trying to log in. These hashes can be generated ahead of time using a tool like https://argon2.online/.',
        'TEXT',
        TRUE,
        TRUE
    );

-- Insert an example usage of the http_header component into the example table
INSERT INTO example (component, description)
VALUES (
        'authentication',
        '

### Usage with HTTP basic authentication

The most basic usage of the authentication component is with the
[`sqlpage.basic_auth_username()`](functions.sql?function=basic_auth_username#function) and
[`sqlpage.basic_auth_password()`](functions.sql?function=basic_auth_password#function) functions.
The component will check if the provided password matches the stored [password hash](/examples/hash_password.sql),
and if not, it will prompt the user to enter a password in a browser popup:

```sql
SELECT ''authentication'' AS component,
    ''$argon2i$v=19$m=8,t=1,p=1$YWFhYWFhYWE$oKBq5E8XFTHO2w'' AS password_hash, -- this is a hash of the password ''password''
    sqlpage.basic_auth_password() AS password; -- this is the password that the user entered in the browser popup
```

You can [generate a password hash using the `hash_password` function](/examples/hash_password.sql).

If you want to have multiple users with different passwords,
you could store them with their password hashes in the database,
or just hardcode them use a `CASE` statement:

```sql
SELECT ''authentication'' AS component,
    case sqlpage.basic_auth_username()
        when ''admin''
            then ''$argon2i$v=19$m=8,t=1,p=1$YWFhYWFhYWE$oKBq5E8XFTHO2w'' -- the password is ''password''
        when ''user''
            then ''$argon2i$v=19$m=8,t=1,p=1$YWFhYWFhYWE$qsrWdjgl96ooYw'' -- the password is ''user''
    end AS password_hash, -- this is a hash of the password ''password''
    sqlpage.basic_auth_password() AS password; -- this is the password that the user entered in the browser popup
```

Try this example online: [SQL Basic Auth](/examples/authentication/basic_auth.sql).

### Advanced user session management

*Basic auth* is the simplest way to password-protect a page,
but it is not very flexible nor user-friendly,
because the browser will show an unstyled popup asking for the username and password.

For more advanced authentication, you can store user information and user sessions in your database.
You can then use the [`form`](components.sql?component=form#component) component to create a custom login form.
When the user submits the form, you check if the password is correct using the `authentication` component.
You then store a unique string of numbers and letters (a session token) both in the user''s browser
using the [`cookie`](components.sql?component=cookie#component) component and in your database.
Then, in all the pages that require authentication, you check if the cookie is present and matches the session token in your database.

You can check if the user has sent the correct password in a form, and if not, redirect them to a login page.

Create a login form in a file called `login.sql`:

```sql
select ''form'' as component, ''Authentication'' as title, ''Log in'' as validate, ''create_session_token.sql'' as action;
select ''Username'' as name, ''admin'' as placeholder;
select ''Password'' as name, ''admin'' as placeholder, ''password'' as type;
```

And then, in `create_session_token.sql` :

```sql
SELECT ''authentication'' AS component,
    ''login.sql'' AS link,
    ''$argon2id$v=19$m=16,t=2,p=1$TERTd0lIcUpraWFTcmRQYw$+bjtag7Xjb6p1dsuYOkngw'' AS password_hash, -- generated using sqlpage.hash_password
    :password AS password; -- this is the password that the user sent through our form

-- The code after this point is only executed if the user has sent the correct password

```

and in `login.sql` :

```sql
SELECT ''form'' AS component, ''Login'' AS title, ''my_protected_page.sql'' AS action;
SELECT ''password'' AS type, ''password'' AS name, ''Password'' AS label;
```

### Advanced: usage with a session token

Calling the `authentication` component is expensive.
The password hashing algorithm is designed to be slow, so that it is difficult to brute-force the password,
even if an attacker gets access to the database.

If you want to avoid calling the `authentication` component on every page, you can use a session token.
A session token is a random string that is generated when the user logs in, and stored in the database.
It has a limited lifetime, and is stored in a cookie in the user''s browser.
When the user visits a page, the session token is sent to the server, and the server checks if it is valid.

```sql
SELECT ''authentication'' AS component,
    ''login.sql'' AS link,
    (SELECT password_hash FROM user WHERE username = :username) AS password_hash,
    :password AS password;

-- The code after this point is only executed if the user has sent the correct password

-- Generate a random session token
INSERT INTO session (id, username)
VALUES (sqlpage.random_string(32), :username)
RETURNING 
    ''cookie'' AS component,
    ''session_token'' AS name,
    id AS value;
```

### Single sign-on with OIDC (OpenID Connect)

If you don''t want to manage your own user database,
you can use OpenID Connect and OAuth2 to authenticate users.
This allows users to log in with their Google, Facebook, or internal company account.

You will find an example of how to do this in the
[Single sign-on with OIDC](https://github.com/sqlpage/SQLPage/tree/main/examples/single%20sign%20on).
');
