---
rows:
  - component: form
    action: examples/show_variables.sql
  - name: Fruit
    type: select
    empty_option: Select a fruit...
    options: '[{"label": "Orange", "value": 0}, {"label": "Apple", "value": 1}, {"label": "Banana", "value":
      3}]'
---

This example illustrates the use of the `select` type.
In this select input, the various options are hardcoded, but they could also be loaded from a database table,
[using a function to convert the rows into a json array](/blog.sql?post=JSON%20in%20SQL%3A%20A%20Comprehensive%20Guide) like
 - `json_group_array()` in SQLite,
 - `json_agg()` in Postgres,
 - `JSON_ARRAYAGG()` in MySQL, or
 - `FOR JSON PATH` in Microsoft SQL Server.


In SQLite, the query would look like
```sql
SELECT
    'select' as type,
    'Select a fruit...' as empty_option,
    'cherry' as prefix_icon,
    json_group_array(json_object(
        'label', name,
        'value', id
    )) as options
FROM fruits
```
