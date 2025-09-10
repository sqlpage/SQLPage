INSERT INTO
    sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES
    (
        'link',
        '0.25.0',
        'link',
        'Returns the URL of a SQLPage file with the given parameters.

### Example

Let''s say you have a database of products, and you want the main page (`index.sql`) to link to the page of each product (`product.sql`) with the product name as a parameter.

In `index.sql`, you can use the `link` function to generate the URL of the product page for each product.

```sql
select ''list'' as component;
select
    name as title,
    sqlpage.link(''product'', json_object(''product_name'', name)) as link
from products;
```

In `product.sql`, you can then use `$product_name` to get the name of the product from the URL parameter:

```sql
select ''hero'' as component, $product_name as title, product_info as description
from products
where name = $product_name;
```

> You could also have manually constructed the URL with `CONCAT(''product?product_name='', name)`,
> but using `sqlpage.link` is better because it ensures that the URL is properly encoded.
> `sqlpage.link` will work even if the product name contains special characters like `&`, while `CONCAT(...)` would break the URL.

### Parameters
 - `file` (TEXT): The name of the SQLPage file to link to.
 - `parameters` (JSON): The parameters to pass to the linked file.
 - `fragment` (TEXT): An optional fragment (hash) to append to the URL. This is useful for linking to a specific section of a page. For instance if `product.sql` contains `select ''text'' as component, ''product_description'' as id;`, you can link to the product description section with `sqlpage.link(''product.sql'', json_object(''product_name'', name), ''product_description'')`.
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
        'link',
        1,
        'file',
        'The path of the SQLPage file to link to, relative to the current file.',
        'TEXT'
    ),
    (
        'link',
        2,
        'parameters',
        'A JSON object with the parameters to pass to the linked file.',
        'JSON'
    ),
    (
        'link',
        3,
        'fragment',
        'An optional fragment (hash) to append to the URL to link to a specific section of the target page.',
        'TEXT'
    );
