## Create a complex API endpoint: the `'contents'` property

You can create an API endpoint that will return a JSON value in any format you want,
to implement a complex API.

You should use [the json functions provided by your database](/blog.sql?post=JSON%20in%20SQL%3A%20A%20Comprehensive%20Guide) to form the value you pass to the `contents` property.
To build a json array out of rows from the database, you can use:
 - `json_group_array()` in SQLite,
 - `json_agg()` in Postgres, or
 - `JSON_ARRAYAGG()` in MySQL.
 - `FOR JSON PATH` in SQL Server.


```sql
SELECT 'json' AS component,
        JSON_OBJECT(
            'users', (
                SELECT JSON_GROUP_ARRAY(
                    JSON_OBJECT(
                        'username', username,
                        'userid', id
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
