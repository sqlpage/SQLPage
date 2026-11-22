---
icon: cookie
since: 0.7.1
category: request-data
summary: Read a named cookie from the incoming request.
return_type: TEXT
parameters:
  - name: name
    type: TEXT
    required: true
    description: The name of the cookie to read.
---

Reads a [cookie](https://en.wikipedia.org/wiki/HTTP_cookie) with the given name from the request.
Returns the value of the cookie as text, or NULL if the cookie is not present.

Cookies can be set using the [cookie component](documentation.sql?component=cookie#component).

### Example

#### Set a cookie

Set a cookie called `username` to greet the user by name every time they visit the page:

```sql
select 'cookie' as component, 'username' as name, :username as value;

SELECT 'form' as component;
SELECT 'username' as name, 'text' as type;
```

#### Read a cookie

Read a cookie called `username` and greet the user by name:

```sql
SELECT 'text' as component,
        'Hello, ' || sqlpage.cookie('username') || '!' as contents;
```
