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

```sql
set request = json_object(
    ''method'', ''POST''
    ''url'', ''https://postman-echo.com/post'',
    ''headers'', json_object(
        ''Content-Type'', ''application/json'',
        ''Authorization'', ''Bearer '' || sqlpage.environment_variable(''MY_API_TOKEN'')
    ),
    ''body'', json_object(
        ''Hello'', ''world'',
    ),
);
set api_results = sqlpage.fetch($request);

select ''code'' as component;
select
    ''API call results'' as title,
    ''json'' as language,
    $api_results as contents;
```

# JSON parameter format

The fetch function accepts either a URL string, or a JSON object with the following parameters:
 - `method`: The HTTP method to use. Defaults to `GET`.
 - `url`: The URL to fetch.
 - `headers`: A JSON object with the headers to send.
 - `body`: The body of the request. If it is a JSON object, it will be sent as JSON. If it is a string, it will be sent as is.
 - `timeout_ms`: The maximum time to wait for the request, in milliseconds. Defaults to 5000.

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
