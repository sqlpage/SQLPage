-- This migration documents the JSON/JSONL response format feature based on HTTP Accept headers

-- Update the json component description to include information about the Accept header feature
UPDATE component 
SET description = 'Converts SQL query results into the JSON machine-readable data format. Ideal to quickly build APIs for interfacing with external systems.
        
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

### Alternative: Using HTTP Accept Headers

SQLPage also supports returning JSON or JSON Lines responses based on the HTTP `Accept` header,
without needing to use this component. This is useful when you want the same SQL file to serve
both HTML pages (for browsers) and JSON data (for API clients).

See [Automatic JSON output based on Accept headers](#example4) for more details.
'
WHERE name = 'json';

-- Add a new example for the Accept header feature
INSERT INTO example (component, description)
VALUES (
    'json',
    '
## Automatic JSON output based on HTTP Accept headers

SQLPage can automatically return JSON or JSON Lines responses instead of HTML based on the HTTP `Accept` header sent by the client.
This allows the same SQL file to serve both web browsers and API clients.

### How it works

When a client sends a request with an `Accept` header, SQLPage checks if the client prefers JSON:

- `Accept: application/json` → Returns a JSON array of all component data
- `Accept: application/x-ndjson` → Returns JSON Lines (one JSON object per line)
- `Accept: text/html` or `Accept: */*` → Returns the normal HTML page

All other SQLPage features work exactly the same:
- Header components (`redirect`, `cookie`, `http_header`, `status_code`, `authentication`) work as expected
- SQLPage functions and variables work normally
- The response just skips HTML template rendering

### Example: A dual-purpose page

The following SQL file works as both a normal web page and a JSON API:

```sql
-- Header components work with both HTML and JSON responses
SELECT ''cookie'' AS component, ''last_visit'' AS name, datetime() AS value;
SELECT ''status_code'' AS component, 200 AS status;

-- These will be rendered as HTML for browsers, or returned as JSON for API clients
SELECT ''text'' AS component, ''Welcome!'' AS contents;
SELECT ''table'' AS component;
SELECT id, name, email FROM users;
```

### HTML Response (default, for browsers)

```html
<!DOCTYPE html>
<html>
  <!-- Normal SQLPage HTML output -->
</html>
```

### JSON Response (when Accept: application/json)

```json
[
  {"component":"text","contents":"Welcome!"},
  {"component":"table"},
  {"id":1,"name":"Alice","email":"alice@example.com"},
  {"id":2,"name":"Bob","email":"bob@example.com"}
]
```

### JSON Lines Response (when Accept: application/x-ndjson)

```
{"component":"text","contents":"Welcome!"}
{"component":"table"}
{"id":1,"name":"Alice","email":"alice@example.com"}
{"id":2,"name":"Bob","email":"bob@example.com"}
```

### Using from JavaScript

```javascript
// Fetch JSON from any SQLPage endpoint
const response = await fetch("/users.sql", {
  headers: { "Accept": "application/json" }
});
const data = await response.json();
console.log(data);
```

### Using from curl

```bash
# Get JSON output
curl -H "Accept: application/json" http://localhost:8080/users.sql

# Get JSON Lines output
curl -H "Accept: application/x-ndjson" http://localhost:8080/users.sql
```

### Comparison with the json component

| Feature | `json` component | Accept header |
|---------|------------------|---------------|
| Use case | Dedicated API endpoint | Dual-purpose page |
| HTML output | Not possible | Default behavior |
| Custom JSON structure | Yes (via `contents`) | No (component data only) |
| Server-sent events | Yes (`type: sse`) | No |
| Requires code changes | Yes | No |
'
);

