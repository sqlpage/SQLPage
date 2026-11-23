---
icon: arrows-shuffle
since: 0.7.2
category: identity-and-security
summary: Generate a cryptographically secure random string.
parameters:
  - name: length
    type: INTEGER
    required: true
    description: The length of the string to generate.
---

Returns a cryptographically secure random string of the given length.

When used as a standalone selected column in a query that returns several rows, `random_string` runs once per returned row.
Use `SET token = sqlpage.random_string(32)` first if you want one token reused later in the page.

### Example

Generate a random string of 32 characters and use it as a session ID stored in a cookie:

```sql
INSERT INTO login_session (session_token, username) VALUES (sqlpage.random_string(32), :username)
RETURNING
    'cookie' AS component,
    'session_id' AS name,
    session_token AS value;
```
