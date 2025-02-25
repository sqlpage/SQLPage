INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'fetch_with_meta',
        '0.34.0',
        'transfer-vertical',
        'Sends an HTTP request and returns detailed metadata about the response, including status code, headers, and body.

This function is similar to [`fetch`](?function=fetch), but returns a JSON object containing detailed information about the response.
The returned object has the following structure:
```json
{
    "status": 200,
    "headers": {
        "content-type": "text/html",
        "content-length": "1234"
    },
    "body": "a string, or a json object, depending on the content type",
    "error": "error message if any"
}
```

If the request fails or encounters an error (e.g., network issues, invalid UTF-8 response), instead of throwing an error,
the function returns a JSON object with an "error" field containing the error message.

### Example: Basic Usage

```sql
-- Make a request and get detailed response information
set response = sqlpage.fetch_with_meta(''https://pokeapi.co/api/v2/pokemon/ditto'');

-- redirect the user to an error page if the request failed
select ''redirect'' as component, ''error.sql'' as url
where
    json_extract($response, ''$.error'') is not null
    or json_extract($response, ''$.status'') != 200;

-- Extract data from the response json body
select ''card'' as component;
select
    json_extract($response, ''$.body.name'') as title,
    json_extract($response, ''$.body.abilities[0].ability.name'') as description
from $response;
```

### Example: Advanced Request with Authentication

```sql
set request = json_object(
    ''method'', ''POST'',
    ''url'', ''https://sqlpage.free.beeceptor.com'',
    ''headers'', json_object(
        ''Content-Type'', ''application/json'',
        ''Authorization'', ''Bearer '' || sqlpage.environment_variable(''API_TOKEN'')
    ),
    ''body'', json_object(
        ''key'', ''value''
    )
);
set response = sqlpage.fetch_with_meta($request);

-- Check response content type
select ''debug'' as component, $response as response;
```

The function accepts the same parameters as the [`fetch` function](?function=fetch).'
    );

INSERT INTO sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES (
        'fetch_with_meta',
        1,
        'url',
        'Either a string containing an URL to request, or a json object in the standard format of the request interface of the web fetch API.',
        'TEXT'
    ); 