---
icon: photo-up
since: 0.33.0
category: request-data
summary: Read a binary request body as a Base64 string.
---

Returns the raw request body encoded in base64. This is useful when receiving binary data or when you need to handle non-text content in your API endpoints.

## Base64 encoding

Base64 is a way to encode binary data (like images or files) into text that can be safely stored and transmitted. This function automatically converts the incoming request body into this format.

## Receiving binary data in an API

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
select 'json' as component,
       json_object(
         'status', 'success',
         'message', 'Image uploaded successfully'
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
>   (in these cases, use [`sqlpage.variables('post')`](?function=variables) instead)
