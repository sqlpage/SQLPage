INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'headers',
        '0.33.0',
        'circle-dotted-letter-h',
        'Returns all HTTP request headers as a JSON object.

### Example

The following displays all HTTP request headers in a list,
using SQLite''s `json_each()` function.

```sql
select ''list'' as component;

select key as title, value as description
from json_each(sqlpage.headers()); -- json_each() is SQLite only
```

If not on SQLite, use your [database''s JSON function](/blog.sql?post=JSON%20in%20SQL%3A%20A%20Comprehensive%20Guide).

### Details

The function returns a JSON object where:
- Keys are lowercase header names
- Values are the corresponding header values
- If no headers are present, returns an empty JSON object `{}`

This is useful when you need to:
- Debug HTTP requests
- Access multiple headers at once

If you only need access to a single known header, use [`sqlpage.header(name)`](?function=header) instead.
');