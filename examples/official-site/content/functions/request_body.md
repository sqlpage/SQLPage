---
icon: http-post
since: 0.33.0
category: request-data
summary: Read the raw request body as text.
---

Returns the raw request body as a string.

A client (like a web browser, mobile app, or another server) can send information to your server in the request body.
This function allows you to read that information in your SQL code,
in order to create or update a resource in your database for instance.

The request body is commonly used when building **REST APIs** (machines-to-machines interfaces)
that receive data from the client.

This is especially useful in:
- `POST` and `PUT` requests for creating or updating resources in your database
- Any API endpoint that needs to receive complex data

### Example: Building a REST API

Here's an example of building an API endpoint that receives a json object,
and inserts it into a database.

#### `api/create_user.sql`
```sql
-- Get the raw JSON body
set user_data = sqlpage.request_body();

-- Insert the user into database
with parsed_data as (
  select
    json_extract($user_data, '$.name') as name,
    json_extract($user_data, '$.email') as email
)
insert into users (name, email)
select name, email from parsed_data;

-- Return success response
select 'json' as component,
       json_object(
         'status', 'success',
         'message', 'User created successfully'
       ) as contents;
```

### Testing the API

You can test this API using curl:
```bash
curl -X POST http://localhost:8080/api/create_user \
  -H "Content-Type: application/json" \
  -d '{"name": "John", "email": "john@example.com"}'
```

## Special cases

### NULL

This function returns NULL if:
 - There is no request body
 - The request content type is `application/x-www-form-urlencoded` or `multipart/form-data`
   (in these cases, use [`sqlpage.variables('post')`](?function=variables) instead)

### Binary data

If the request body is not valid text encoded in UTF-8,
invalid characters are replaced with the Unicode replacement character `�` (U+FFFD).

If you need to handle binary data,
use [`sqlpage.request_body_base64()`](?function=request_body_base64) instead.
