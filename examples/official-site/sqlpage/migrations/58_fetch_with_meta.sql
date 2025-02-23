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

This function is similar to `fetch`, but returns a JSON object containing detailed information about the response.
The returned object has the following structure:
```json
{
    "status": 200,
    "headers": {
        "content-type": "application/json",
        "content-length": "1234",
        ...
    },
    "body": "response body content",
    "error": "error message if any"
}
```

If the request fails or encounters an error (e.g., network issues, invalid UTF-8 response), instead of throwing an error,
the function returns a JSON object with an "error" field containing the error message.

### Example: Basic Usage

```sql
-- Make a request and get detailed response information
set response = sqlpage.fetch_with_meta(''https://api.example.com/data'');

-- Check if the request was successful
select case 
    when json_extract($response, ''$.error'') is not null then
        ''Request failed: '' || json_extract($response, ''$.error'')
    when json_extract($response, ''$.status'') != 200 then
        ''Request returned status '' || json_extract($response, ''$.status'')
    else
        ''Request successful''
end as message;

-- Display response headers
select ''code'' as component,
    ''Response Headers'' as title,
    ''json'' as language,
    json_extract($response, ''$.headers'') as contents;
```

### Example: Error Handling with Retries

```sql
-- Function to make a request with retries
create temp table if not exists make_request as
with recursive retry(attempt, response) as (
    -- First attempt
    select 1 as attempt,
           sqlpage.fetch_with_meta(''https://api.example.com/data'') as response
    union all
    -- Retry up to 3 times if we get a 5xx error
    select attempt + 1,
           sqlpage.fetch_with_meta(''https://api.example.com/data'')
    from retry
    where attempt < 3
    and (
        json_extract(response, ''$.error'') is not null
        or cast(json_extract(response, ''$.status'') as integer) >= 500
    )
)
select response
from retry
where json_extract(response, ''$.error'') is null
  and cast(json_extract(response, ''$.status'') as integer) < 500
limit 1;

-- Use the response
select case
    when $response is null then ''All retry attempts failed''
    else ''Request succeeded after retries''
end as message;
```

### Example: Advanced Request with Authentication

```sql
set request = json_object(
    ''method'', ''POST'',
    ''url'', ''https://api.example.com/data'',
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
select case
    when json_extract($response, ''$.headers.content-type'') like ''%application/json%''
    then json_extract($response, ''$.body'')
    else null
end as json_response;
```

The function accepts the same parameters as the `fetch` function. See the documentation of `fetch` for more details about the available parameters.'
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