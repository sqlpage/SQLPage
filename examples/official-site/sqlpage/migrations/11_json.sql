INSERT INTO
    component (name, description, icon, introduced_in_version)
VALUES
    (
        'json',
        'For advanced users, allows you to easily build an API over your database.
        The json component responds to the current HTTP request with a JSON object.
        This component must appear at the top of your SQL file, before any other data has been sent to the browser.',
        'code',
        '0.9.0'
    );

-- Insert the parameters for the http_header component into the parameter table
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
        'json',
        'contents',
        'A single JSON payload to send. You can use your database''s built-in json functions to build the value to enter here. If not provided, the contents will be taken from the next SQL statements and rendered as a JSON array.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'json',
        'type',
        'The type of the JSON payload to send. Defaults to "array" (each query result is rendered as a JSON object in the array). Other possible values are "jsonlines" (each query result is rendered as a JSON object in a new line, without a top-level array) and "sse" (each query result is rendered as a JSON object in a new line, prefixed by "data: ", which allows you to read the results as server-sent events in real-time from javascript).',
        'TEXT',
        TRUE,
        TRUE
    );

-- Insert an example usage of the http_header component into the example table
INSERT INTO
    example (component, description)
VALUES
    (
        'json',
        '
## Send query results as a JSON array

### SQL

```sql
select ''json'' AS component;
select * from users;
```

### Result

```json
[
    {"username":"James","userid":1},
    {"username":"John","userid":2}
]
```
    '
    ),
    (
        'json',
        '
## Send a single JSON object

### SQL

```sql
select ''json'' AS component, ''jsonlines'' AS type;
select * from users where id = $user_id LIMIT 1;
```

### Result

```json
{ "username":"James", "userid":1 }
```
'
    ),
    (
        'json',
        '
## Create a complex API endpoint

This will create an API endpoint that will allow developers to easily query a list of users stored in your database.

You should use [the json functions provided by your database](/blog.sql?post=JSON%20in%20SQL%3A%20A%20Comprehensive%20Guide) to form the value you pass to the `contents` property.
To build a json array out of rows from the database, you can use: 
 - `json_group_array()` in SQLite,
 - `json_agg()` in Postgres, or
 - `JSON_ARRAYAGG()` in MySQL.
 - `FOR JSON PATH` in SQL Server.


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

If you want to handle custom API routes, like `POST /api/users/:id`,
you can use 
 - the [`404.sql` file](/your-first-sql-website/custom_urls.sql) to handle the request despite the URL not matching any file,
 - the [`request_method` function](/functions.sql?function=request_method#function) to differentiate between GET and POST requests,
 - and the [`path` function](/functions.sql?function=path#function) to extract the `:id` parameter from the URL.
'
    ),
    (
        'json',
        '
## Access query results in real-time with server-sent events

Using server-sent events, you can stream query results to the client in real-time.
This means you can build dynamic applications that will process data as it arrives.

### SQL

```sql
select ''json'' AS component, ''sse'' AS type;
select * from users;
```

### JavaScript

```javascript
const eventSource = new EventSource("users.sql");
eventSource.onmessage = function (event) {
    const user = JSON.parse(event.data);
    console.log(user.username);
}
```
'
    );
