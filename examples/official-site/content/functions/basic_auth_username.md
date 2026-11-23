---
icon: user
since: 0.7.2
category: identity-and-security
summary: Read the username sent with HTTP Basic authentication.
return_type: TEXT
---

Returns the username from the [Basic Authentication](https://en.wikipedia.org/wiki/Basic_access_authentication) header of the request.
        If the header is not present, this function raises an authorization error that will prompt the user to enter their credentials.

### Example

```sql
SELECT 'authentication' AS component,
    (SELECT password_hash from users where name = sqlpage.basic_auth_username()) AS password_hash,
    sqlpage.basic_auth_password() AS password;
```
