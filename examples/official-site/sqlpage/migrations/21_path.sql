INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'path',
        '0.15.0',
        'slashes',
        'Returns the request path of the current page.
This is useful to generate links to the current page, and when you have a proxy in front of your SQLPage server that rewrites the URL.

### Example

If we have a page in a file named `my page.sql` at the root of your SQLPage installation
then the following SQL query:

```sql
select ''text'' as component, sqlpage.path() as contents;
```

will return `/my%20page.sql`.

> Note that the path is URL-encoded.
');