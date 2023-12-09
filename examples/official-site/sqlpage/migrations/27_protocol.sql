INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'protocol',
        '0.17.1',
        'network',
        'Returns the protocol that was used to access the current page.

This can be either `http` or `https`.

This is useful to generate links to the current page.

### Example

```sql
select ''text'' as component,
        sqlpage.protocol() || ''://'' || sqlpage.header(''host'') || sqlpage.path() as contents;
```

will return `https://example.com/example.sql`.

> Note that the path is URL-encoded. The protocol is resolved in this order:
> - `Forwarded` header
> - `X-Forwarded-Proto` header
> request target / URI
');