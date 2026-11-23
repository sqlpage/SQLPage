---
icon: network
since: 0.17.1
category: request-data
summary: Identify whether the request used HTTP or HTTPS.
---

Returns the protocol that was used to access the current page.

This can be either `http` or `https`.

This is useful to generate links to the current page.

### Example

```sql
select 'text' as component,
        sqlpage.protocol() || '://' || sqlpage.header('host') || sqlpage.path() as contents;
```

will return `https://example.com/example.sql`.

> Note that the path is URL-encoded. The protocol is resolved in this order:
> - `Forwarded` header
> - `X-Forwarded-Proto` header
> request target / URI
