---
icon: code
since: 0.9.0
category: http-responses
summary: Return query results as JSON for an API.
top:
  - name: contents
    type: TEXT
    description: A single JSON payload to send. You can use your database's built-in json functions to build
      the value to enter here. If not provided, the contents will be taken from the next SQL statements and
      rendered as a JSON array.
  - name: type
    type: TEXT
    description: |-
      The type of the JSON payload to send: "array", "jsonlines", or "sse".
      In "array" mode, each query result is rendered as a JSON object in a single top-level array.
      In "jsonlines" mode, results are rendered as JSON objects in separate lines, without a top-level array.
      In "sse" mode, results are rendered as JSON objects in separate lines, prefixed by "data: ", which allows you to read the results as server-sent events in real-time from javascript.
---

Converts SQL query results into the JSON machine-readable data format. Ideal to quickly build APIs for interfacing with external systems.

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
