---
rows:
  - component: form
    action: examples/show_variables.sql
    reset: Reset
  - name: component
    type: select
    value: form
    options:
      - label: Form
        value: form
    options_source: examples/from_component_options_source.sql?category=component
    description: Start typing the name of a component like 'map' or 'form'...
---

### Multi-select
You can authorize the user to select multiple options by setting the `multiple` property to `true`.
This creates a more compact (but arguably less user-friendly) alternative to a series of checkboxes.
In this case, you should add square brackets to the name of the field (e.g. `'my_field[]' as name`).
The target page will then receive the value as a JSON array of strings, which you can iterate over using
 - the `json_each` function [in SQLite](https://www.sqlite.org/json1.html) and [Postgres](https://www.postgresql.org/docs/9.3/functions-json.html),
 - the [`OPENJSON`](https://learn.microsoft.com/fr-fr/sql/t-sql/functions/openjson-transact-sql?view=sql-server-ver16) function in Microsoft SQL Server.
 - in MySQL, json manipulation is less straightforward: see [the SQLPage MySQL json example](https://github.com/sqlpage/SQLPage/tree/main/examples/mysql%20json%20handling)

[More information on how to handle JSON in SQL](/blog.sql?post=JSON%20in%20SQL%3A%20A%20Comprehensive%20Guide).

The target page could then look like this:

```sql
insert into best_fruits(id) -- INSERT INTO ... SELECT ... runs the SELECT query and inserts the results into the table
select CAST(value AS integer) as id -- all values are transmitted by the browser as strings
from json_each($my_field); -- in SQLite, json_each returns a table with a "value" column for each element in the JSON array
```

### Example multiselect generated from a database table

If you have a table of all possible options (`my_options(id int, label text)`),
and want to generate a multi-select field from it, you have two options:
- if the number of options is not too large, you can use the `options` parameter to return them all as a JSON array in the SQL query
- if the number of options is large (e.g. more than 1000), you can use `options_source` to load options dynamically from a different SQL query as the user types

#### Embedding all options in the SQL query

Let's say you have a table that contains the selected options per user (`my_user_options(user_id int, option_id int)`).
You can use a query like this to generate the multi-select field:

```sql
select 'select' as type, true as multiple, json_group_array(json_object(
    'label', my_options.label,
    'value', my_options.id,
    'selected', my_user_options.option_id is not null
)) as options
from my_options
left join my_user_options
    on  my_options.id = my_user_options.option_id
    and my_user_options.user_id = $user_id
```

This will generate a json array of objects, each containing the label, value and selected status of each option.

#### Loading options dynamically from a different SQL query with `options_source`

If the `my_options` table has a large number of rows, you can use the `options_source` parameter to load options dynamically from a different SQL query as the user types.

We'll write a second SQL file, `options_source.sql`, that will receive the user's search string as a parameter named `$search`,
 and return a json array of objects, each containing the label and value of each option.

When both `options` and `options_source` are set, the local `options` are loaded first. Search results from `options_source` are loaded into the same option list; a result with the same `value` updates the existing option. This is useful to set a default preselected value with `options_source`.

`options_source` may already contain query parameters. SQLPage preserves them and adds or replaces the `search` parameter when loading options.

##### `options_source.sql`

```sql
select 'json' as component;

select id as value, label as label
from my_options
where label like $search || '%';
```

##### `form`
