INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'redirect',
        'Redirects the user to another page.
        This component is useful for implementing redirects after a form submission,
        or to redirect users to a login page if they are not logged in.
        
        Contrary to the http_header component, this component completely stops the execution of the page after it is called,
        so it is suitable to use to hide sensitive information from users that are not logged in, for example.

        Since it uses an HTTP header to redirect the user, it is not possible to use this component after the page has started being sent to the browser.',
        'arrow-right',
        '0.7.2'
    );
-- Insert the parameters for the redirect component into the parameter table
INSERT INTO parameter (
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
VALUES (
        'redirect',
        'link',
        'The URL to redirect the user to.',
        'TEXT',
        TRUE,
        FALSE
    );
-- Insert an example usage of the redirect component into the example table
INSERT INTO example (component, description)
VALUES (
        'redirect',
        '
Redirect a user to the login page if they are not logged in:

```sql
SELECT ''redirect'' AS component, ''login.sql'' AS link
WHERE NOT EXISTS (SELECT 1 FROM login_session WHERE id = sqlpage.cookie(''session_id''));
```
'
    );