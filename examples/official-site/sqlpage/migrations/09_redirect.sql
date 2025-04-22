INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'redirect',
        'Redirects the user to another page.

This component helps you:
1. Send users to a different page
1. Stop execution of the current page

### Conditional logic

There is no `IF` statement in SQL. Even when you use a [`CASE` expression](https://modern-sql.com/caniuse/case_(simple)), all branches are always evaluated (and only one is returned).

To conditionally execute a component or a [SQLPage function](/functions.sql), you can use the `redirect` component.
A common use case is error handling. You may want to proceed with the rest of a page only when certain pre-conditions are met.

```sql
SELECT
    ''redirect'' AS component,
    ''error_page.sql'' AS link
WHERE NOT your_condition;

-- The rest of the page is only executed if the condition is true
```
### Technical limitation

You must use this component **at the beginning of your SQL file**, before any other components that might send content to the browser.
Since the component needs to tell the browser to go to a different page by sending an *HTTP header*,
it will fail if the HTTP headers have already been sent by the time it is executed.

> **Important difference from [http_header](?component=http_header)**
>
> This component completely stops the page from running after it''s called.
> This makes it a good choice for protecting sensitive information from unauthorized users.

',
        'arrow-right',
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
        'redirect',
        'link',
        'The URL to redirect the user to.',
        'TEXT',
        TRUE,
        FALSE
    );
-- Insert an example usage of the http_header component into the example table
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