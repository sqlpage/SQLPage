---
icon: error-404
category: http-responses
summary: Set the HTTP status code returned by a page.
top:
  - name: status
    type: INTEGER
    required: true
    description: HTTP status code (e.g., 200 OK, 401 Unauthorized, 409 Conflict)
---

Sets the HTTP response code for the current page.

This is an advanced technical component.
You typically need it when building internet-facing APIs and websites,
but you may not need it for simple internal applications.

- Indicating operation results when using [SQLPage as an API](?component=json)
  - `200`: *OK*, for successful operations
  - `201`: *Created*, for successful record insertion
  - `404`: *Not Found*, for missing resources
  - `500`: *Internal Server Error*, for failed operations
- Handling data validation errors
  - `400`: *Bad Request*, for invalid data
- Enforcing access controls
  - `403`: *Forbidden*, for unauthorized access
  - `401`: *Unauthorized*, for unauthenticated access
- Tracking system health
  - `500`: *Internal Server Error*, for failed operations

For search engine optimization:
- Use `404` for deleted content to remove outdated URLs from search engines
- For redirection from one page to another, use
  - `301` (moved permanently), or
  - `302` (moved temporarily)
- Use `503` during maintenance
