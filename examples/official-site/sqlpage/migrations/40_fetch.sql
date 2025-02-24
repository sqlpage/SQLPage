INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'fetch',
        '0.20.3',
        'transfer-vertical',
        'Sends an HTTP request and returns the results as a string.

### Example

#### Simple GET query

In this example, we use an API call to find the latitude and longitude of a place
the user searched for, and we display it on a map.

We use the simplest form of the fetch function, that takes the URL to fetch as a string.


```sql
set url = ''https://nominatim.openstreetmap.org/search?format=json&q='' || sqlpage.url_encode($user_search)
set api_results = sqlpage.fetch($url);

select ''map'' as component;
select $user_search as title,
  CAST($api_results->>0->>''lat'' AS FLOAT) as latitude,
  CAST($api_results->>0->>''lon'' AS FLOAT) as longitude;
```

#### POST query with a body

In this example, we use the complex form of the function to make an
authenticated POST request, with custom request headers and a custom request body.

We use SQLite''s json functions to build the request body.
See [the list of SQL databases and their JSON functions](/blog.sql?post=JSON%20in%20SQL%3A%20A%20Comprehensive%20Guide) for 
more information on how to build JSON objects in your database.

```sql
set request = json_object(
    ''method'', ''POST'',
    ''url'', ''https://postman-echo.com/post'',
    ''headers'', json_object(
        ''Content-Type'', ''application/json'',
        ''Authorization'', ''Bearer '' || sqlpage.environment_variable(''MY_API_TOKEN'')
    ),
    ''body'', json_object(
        ''Hello'', ''world''
    )
);
set api_results = sqlpage.fetch($request);

select ''code'' as component;
select
    ''API call results'' as title,
    ''json'' as language,
    $api_results as contents;
```


#### Authenticated request using Basic Auth

Here''s how to make a request to an API that requires [HTTP Basic Authentication](https://en.wikipedia.org/wiki/Basic_access_authentication):

```sql
set request = json_object(
    ''url'', ''https://api.example.com/data'',
    ''username'', ''my_username'',
    ''password'', ''my_password''
);
set api_results = sqlpage.fetch($request);
```

> This will add the `Authorization: Basic bXlfdXNlcm5hbWU6bXlfcGFzc3dvcmQK` header to the request,
> where `bXlfdXNlcm5hbWU6bXlfcGFzc3dvcmQK` is the base64 encoding of the string `my_username:my_password`.

# JSON parameter format

The fetch function accepts either a URL string, or a JSON object with the following parameters:
 - `url`: The URL to fetch. Required.
 - `method`: The HTTP method to use. Defaults to `GET`.
 - `headers`: A JSON object with the headers to send. Defaults to sending a User-Agent header containing the SQLPage version.
 - `body`: The body of the request. If it is a JSON object, it will be sent as JSON. If it is a string, it will be sent as is. When omitted, no request body is sent.
 - `timeout_ms`: The maximum time to wait for the request, in milliseconds. Defaults to 5000.
 - `username`: Optional username for HTTP Basic Authentication. Introduced in version 0.33.0.
 - `password`: Optional password for HTTP Basic Authentication. Only used if username is provided. Introduced in version 0.33.0.

# Error handling and reading response headers

If the request fails, this function throws an error, that will be displayed to the user.
The response headers are not available for inspection.

If you need to handle errors or inspect the response headers, use [`sqlpage.fetch_with_meta`](?function=fetch_with_meta).
'
    );
INSERT INTO sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES (
        'fetch',
        1,
        'url',
        'Either a string containing an URL to request, or a json object in the standard format of the request interface of the web fetch API.',
        'TEXT'
    );
