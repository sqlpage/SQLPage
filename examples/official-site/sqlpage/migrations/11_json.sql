INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'json',
        'For advanced users, allows you to easily build an API over your database.
        The json component responds to the current HTTP request with a JSON object.
        This component must appear at the top of your SQL file, before any other data has been sent to the browser.',
        'code',
        '0.9.0'
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
        'json',
        'contents',
        'The JSON payload to send. You should use your database''s built-in json functions to build the value to enter here.',
        'TEXT',
        TRUE,
        FALSE
    );
-- Insert an example usage of the http_header component into the example table
INSERT INTO example (component, description)
VALUES (
        'json',
        '
Creates an API endpoint that will allow developers to easily query a list of users stored in your database.

You should use the json functions provided by your database to form the value you pass to the `contents` property.
To build a json array out of rows from the database, you can use: 
 - `json_group_array()` in SQLite,
 - `json_agg()` in Postgres, or
 - `JSON_ARRAYAGG()` in MySQL.


```sql
SELECT ''json'' AS component, 
        JSON_OBJECT(
            ''users'', (
                SELECT JSON_GROUP_ARRAY(
                    JSON_OBJECT(
                        ''username'', username,
                        ''userid'', id
                    )
                ) FROM users
            )
        ) AS contents;
```

This will return a JSON response that looks like this:

```json
{ 
    "users" : [
        { "username":"James", "userid":1 }
    ]
}
```

'
    );