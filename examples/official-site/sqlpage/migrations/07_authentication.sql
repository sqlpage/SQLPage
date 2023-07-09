-- Insert the http_header component into the component table
INSERT INTO component (name, description, icon)
VALUES (
        'authentication',
        'An advanced component that can be used to create pages with password-restricted access.
        When used, this component has to be at the top of your page, because once the page has begun being sent to the browser, it is too late to restrict access to it.
        The authentication component checks if the user has sent the correct password, and if not, redirects them to the URL specified in the link parameter.
        If you don''t want to re-check the password on every page (which is an expensive operation),
        you can check the password only once and store a session token in your database. 
        You can use the cookie component to set the session token cookie in the client browser,
        and then check whether the token matches what you stored in subsequent pages.',
        'lock'
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
        'The URL to redirect the user to if they are not logged in.',
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
The most basic usage of the authentication component is to simply check if the user has sent the correct password, and if not, redirect them to a login page: 

```sql
SELECT ''authentication'' AS component,
    ''/login'' AS link,
    ''$argon2id$v=19$m=16,t=2,p=1$TERTd0lIcUpraWFTcmRQYw$+bjtag7Xjb6p1dsuYOkngw'' AS password_hash, -- generated using https://argon2.online/
    :password AS password; -- this is the password that the user sent through our form
```

and in `login.sql` :

```sql
SELECT ''form'' AS component, ''Login'' AS title, ''my_protected_page.sql'' AS action;
SELECT ''password'' AS type, ''password'' AS name, ''Password'' AS label;
```
');