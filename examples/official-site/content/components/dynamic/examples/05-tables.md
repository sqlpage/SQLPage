## Dynamic tables

The `dynamic` component can be used to generate [tables](?component=table#component) with dynamic columns,
using [your database's JSON functions](/blog.sql?post=JSON%20in%20SQL%3A%20A%20Comprehensive%20Guide).

For instance, let's say we have a table with three columns: user_id, name, and role.
We want to create a table where each row is a user, and each column is a role.
We will return a set of json objects that look like this: `{"name": "Alice", "admin": true, "editor": false, "viewer": true}`
```sql
SELECT 'table' AS component;
SELECT 'dynamic' AS component,
    json_patch(
        json_object('name', name),
        json_object_agg(role, is_admin)
    ) AS properties
FROM users
GROUP BY name;
```
