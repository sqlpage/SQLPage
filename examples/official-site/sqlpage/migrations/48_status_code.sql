-- Insert the status_code component into the component table
INSERT INTO
    component (name, description, icon)
VALUES
    (
        'status_code',
        'A simple component to set the HTTP status code for the response. This can be used to indicate the result of the request, such as 200 for success, 404 for not found, or 500 for server error.

        The status code should be set according to the HTTP standard status codes.

        This component should be used when you need to explicitly set the status code of the HTTP response.
        ',
        'error-404'
    );

-- Insert the parameters for the status_code component into the parameter table
INSERT INTO
    parameter (
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
VALUES
    (
        'status_code',
        'status',
        'The HTTP status code to be set for the response. This should be an integer value representing a valid HTTP status code.',
        'INTEGER',
        TRUE,
        FALSE
    );


INSERT INTO example (component, description)
VALUES (
        'status_code',
        'Set the HTTP status code to 404, indicating that the requested resource was not found.
Useful in combination with [`404.sql` files](/your-first-sql-website/custom_urls.sql):

```sql
select ''status_code'' as component, 404 as status;
```
');
