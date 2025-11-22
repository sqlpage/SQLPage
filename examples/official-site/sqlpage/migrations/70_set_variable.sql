INSERT INTO
    sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES
    (
        'set_variable',
        '0.40.0',
        'variable',
        'Returns a URL that is the same as the current page''s URL, but with a variable set to a new value.
        
This function is useful when you want to create a link that changes a parameter on the current page, while preserving other parameters.

It is equivalent to `sqlpage.link(sqlpage.path(), json_patch(sqlpage.variables(''get''), json_object(name, value)))`.

### Example

Let''s say you have a list of products, and you want to filter them by category. You can use `sqlpage.set_variable` to create links that change the category filter, without losing other potential filters (like a search query or a sort order).

```sql
select ''button'' as component, ''sm'' as size, ''center'' as justify;
select 
    category as title,
    sqlpage.set_variable(''category'', category) as link,
    case when $category = category then ''primary'' else ''secondary'' end as color
from categories;
```

### Parameters
 - `name` (TEXT): The name of the variable to set.
 - `value` (TEXT): The value to set the variable to. If `NULL` is passed, the variable is removed from the URL.
'
    );

INSERT INTO
    sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES
    (
        'set_variable',
        1,
        'name',
        'The name of the variable to set.',
        'TEXT'
    ),
    (
        'set_variable',
        2,
        'value',
        'The value to set the variable to.',
        'TEXT'
    );
