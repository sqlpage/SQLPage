---
icon: variable
since: 0.40.0
category: links-and-routing
summary: Build a link to this page with a query parameter changed or removed.
parameters:
  - name: name
    type: TEXT
    required: true
    description: The name of the variable to set.
  - name: value
    type: TEXT
    description: The value to set the variable to.
---

Returns a URL that is the same as the current page's URL, but with a variable set to a new value.

This function is useful when you want to create a link that changes a parameter on the current page, while preserving other parameters.

It is equivalent to `sqlpage.link(sqlpage.path(), json_patch(sqlpage.variables('get'), json_object(name, value)))`.

### Example

Let's say you have a list of products, and you want to filter them by category. You can use `sqlpage.set_variable` to create links that change the category filter, without losing other potential filters (like a search query or a sort order).

```sql
select 'button' as component, 'sm' as size, 'center' as justify;
select
    category as title,
    sqlpage.set_variable('category', category) as link,
    case when $category = category then 'primary' else 'secondary' end as color
from categories;
```

When `sqlpage.set_variable(...)` is used as a standalone selected column in a query that returns several rows, it runs once per returned row.
This is useful for generating one link per row, as in the example above.
If you need a single link reused later in the page, store it with `SET` first.

### Parameters
 - `name` (TEXT): The name of the variable to set.
 - `value` (TEXT): The value to set the variable to. If `NULL` is passed, the variable is removed from the URL.
