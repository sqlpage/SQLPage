---
namespace: sqlpage
return_type: TEXT
introduced_in_version: "0.1.0"
category: http
difficulty: beginner
---

# cookie Function

## Signature

```sql
sqlpage.cookie(name TEXT) -> TEXT
```

## Description

The `cookie` function retrieves the value of a cookie from the HTTP request. This is useful for accessing user preferences, session data, or other client-side stored information.

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| name | TEXT | Yes | The name of the cookie to retrieve |

## Return Value

Returns the value of the specified cookie as TEXT, or NULL if the cookie does not exist.

## Security Notes

- Cookie values are provided by the client and should not be trusted without validation
- Always validate and sanitize cookie values before using them in SQL queries
- Consider using signed or encrypted cookies for sensitive data

## Examples

### Basic Cookie Retrieval

```sql
SELECT 'text' AS component, 'Welcome back, ' || sqlpage.cookie('username') AS contents;
```

### Cookie with Default Value

```sql
SELECT 'text' AS component, 
       COALESCE(sqlpage.cookie('theme'), 'light') AS contents;
```

### User Preferences

```sql
SELECT 'text' AS component, 
       CASE 
         WHEN sqlpage.cookie('language') = 'es' THEN 'Hola'
         WHEN sqlpage.cookie('language') = 'fr' THEN 'Bonjour'
         ELSE 'Hello'
       END AS contents;
```

### Session Management

```sql
-- Check if user is logged in
SELECT 'text' AS component,
       CASE 
         WHEN sqlpage.cookie('session_id') IS NOT NULL THEN 'Welcome back!'
         ELSE 'Please log in'
       END AS contents;
```

## Related

- [set_cookie Function](./set_cookie.md)
- [Session Management Guide](../guides/session-management.md)
- [User Authentication Guide](../guides/user-authentication.md)