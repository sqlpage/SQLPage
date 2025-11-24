INSERT INTO
    component (name, description, icon, introduced_in_version)
VALUES
    (
        'json',
        'Converts SQL query results into the JSON machine-readable data format. Ideal to quickly build APIs for interfacing with external systems.
        
**JSON** is a widely used data format for programmatic data exchange.
For example, you can use it to integrate with web services written in different languages,
with mobile or desktop apps, or with [custom client-side components](/custom_components.sql) inside your SQLPage app.

Use it when your application needs to expose data to external systems.
If you only need to render standard web pages,
and do not need other software to access your data,
you can ignore this component.

This component **must appear at the top of your SQL file**, before any other data has been sent to the browser.
An HTTP response can have only a single datatype, and it must be declared in the headers.
So if you have already called the `shell` component, or another traditional HTML component,
you cannot use this component in the same file.

SQLPage can also return JSON or JSON Lines when the incoming request says it prefers them with an HTTP `Accept` header, so the same `/users.sql` page can show a table in a browser but return raw data to `curl -H "Accept: application/json" http://localhost:8080/users.sql`.

Use this component when you want to control the payload or force JSON output even for requests that would normally get HTML.
',
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
        'The type of the JSON payload to send: "array", "jsonlines", or "sse".
In "array" mode, each query result is rendered as a JSON object in a single top-level array.
In "jsonlines" mode, results are rendered as JSON objects in separate lines, without a top-level array.
In "sse" mode, results are rendered as JSON objects in separate lines, prefixed by "data: ", which allows you to read the results as server-sent events in real-time from javascript.',
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
## Send query results as a single JSON array: `''array'' as type`

The default `array` mode sends the query results as a single JSON array.

If a query returns an error, the array will contain an object with an `error` property.

If multiple queries are executed, all query results will be concatenated into a single array
of heterogeneous objects.

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

Clients can also receive JSON or JSON Lines automatically by requesting the same SQL file with an HTTP `Accept` header such as `application/json` or `application/x-ndjson` when the component is omitted, for example:

```
curl -H "Accept: application/json" http://localhost:8080/users.sql
```
    '
    ),
    (
        'json',
        '
## Send a single JSON object: `''jsonlines'' as type`

In `jsonlines` mode, each query result is rendered as a JSON object in a separate line,
without a top-level array.

If there is a single query result, the response will be a valid JSON object.
If there are multiple query results, you will need to parse each line of the response as a separate JSON object.

If a query returns an error, the response will be a JSON object with an `error` property.

### SQL

The following SQL creates an API endpoint that takes a `user_id` URL parameter
and returns a single JSON object containing the user''s details, with one json object key per column in the `users` table.

```sql
select ''json'' AS component, ''jsonlines'' AS type;
select * from users where id = $user_id LIMIT 1;
```

> Note the `LIMIT 1` clause. The `jsonlines` type will send one JSON object per result row,
> separated only by a single newline character (\n).
> So if your query returns multiple rows, the result will not be a single valid JSON object,
> like most JSON parsers expect.

### Result

```json
{ "username":"James", "userid":1 }
```
'
    ),
    (
        'json',
        '
## Create a complex API endpoint: the `''contents''` property

You can create an API endpoint that will return a JSON value in any format you want,
to implement a complex API.

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
## Access query results in real-time with server-sent events: `''sse'' as type`

Using server-sent events, you can stream large query results to the client in real-time,
row by row.

This allows building sophisticated dynamic web applications that will start processing and displaying 
the first rows of data in the browser while the database server is still processing the end of the query.

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
eventSource.onerror = () => eventSource.close(); // do not reconnect after reading all the data
```
'
    );
