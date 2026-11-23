---
icon: heading
since: 0.7.2
category: request-data
summary: Read the value of one HTTP request header.
parameters:
  - name: name
    type: TEXT
    required: true
    description: The name of the HTTP header to read.
---

Reads a [header](https://en.wikipedia.org/wiki/List_of_HTTP_header_fields) with the given name from the request.
    Returns the value of the header as text, or NULL if the header is not present.

### Example

Log the [`User-Agent`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent) of the browser making the request in the database:

```sql
INSERT INTO user_agent_log (user_agent) VALUES (sqlpage.header('user-agent'));
```

If you need access to all headers at once, use [`sqlpage.headers()`](?function=headers) instead.
