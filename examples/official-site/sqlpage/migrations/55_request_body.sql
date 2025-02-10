INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'request_body',
        '0.33.0',
        'http-post',
        'Returns the raw request body as a string.

A client (like a web browser, mobile app, or another server) can send information to your server in the request body.
This function allows you to read that information in your SQL code,
in order to create or update a resource in your database for instance.

The request body is commonly used when building **REST APIs** (machines-to-machines interfaces)
that receive data from the client.

This is especially useful in:
- `POST` and `PUT` requests for creating or updating resources in your database
- Any API endpoint that needs to receive complex data

### Example: Building a REST API

Here''s an example of building an API endpoint that receives a json object,
and inserts it into a database.

#### `api/create_user.sql`
```sql
-- Get the raw JSON body
set user_data = sqlpage.request_body();

-- Insert the user into database
with parsed_data as (
  select 
    json_extract($user_data, ''$.name'') as name,
    json_extract($user_data, ''$.email'') as email
)
insert into users (name, email)
select name, email from parsed_data;

-- Return success response
select ''json'' as component,
       json_object(
         ''status'', ''success'',
         ''message'', ''User created successfully''
       ) as contents;
```

### Testing the API

You can test this API using curl:
```bash
curl -X POST http://localhost:8080/api/create_user \
  -H "Content-Type: application/json" \
  -d ''{"name": "John", "email": "john@example.com"}''
```

## Special cases

### NULL

This function returns NULL if:
 - There is no request body
 - The request content type is `application/x-www-form-urlencoded` or `multipart/form-data` 
   (in these cases, use [`sqlpage.variables(''post'')`](?function=variables) instead)

### Binary data

If the request body is not valid text encoded in UTF-8,
invalid characters are replaced with the Unicode replacement character `�` (U+FFFD).

If you need to handle binary data,
use [`sqlpage.request_body_base64()`](?function=request_body_base64) instead.
'
    );

INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'request_body_base64',
        '0.33.0',
        'photo-up',
        'Returns the raw request body encoded in base64. This is useful when receiving binary data or when you need to handle non-text content in your API endpoints.

### What is Base64?

Base64 is a way to encode binary data (like images or files) into text that can be safely stored and transmitted. This function automatically converts the incoming request body into this format.

### Example: Handling Binary Data in an API

This example shows how to receive and process an image uploaded directly in the request body:

```sql
-- Assuming this is api/upload_image.sql
-- Client would send a POST request with the raw image data

-- Get the base64-encoded image data
set image_data = sqlpage.request_body_base64();

-- Store the image data in the database
insert into images (data, uploaded_at)
values ($image_data, current_timestamp);

-- Return success response
select ''json'' as component,
       json_object(
         ''status'', ''success'',
         ''message'', ''Image uploaded successfully''
       ) as contents;
```

You can test this API using curl:
```bash
curl -X POST http://localhost:8080/api/upload_image.sql \
  -H "Content-Type: application/octet-stream" \
  --data-binary "@/path/to/image.jpg"
```

This is particularly useful when:
- Working with binary data (images, files, etc.)
- The request body contains non-UTF8 characters
- You need to pass the raw body to another system that expects base64

> Note: Like [`sqlpage.request_body()`](?function=request_body), this function returns NULL if:
> - There is no request body
> - The request content type is `application/x-www-form-urlencoded` or `multipart/form-data`
>   (in these cases, use [`sqlpage.variables(''post'')`](?function=variables) instead)
'
    );