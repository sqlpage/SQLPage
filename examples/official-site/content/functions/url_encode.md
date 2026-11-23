---
icon: percentage
since: 0.12.0
category: links-and-routing
summary: Encode text for use in a URL.
parameters:
  - name: string
    type: TEXT
    required: true
    description: The string to encode.
---

Returns the given string, with all characters that are not allowed in a URL encoded.

### Example

```sql
select 'text' as component;
select 'https://example.com/?q=' || sqlpage.url_encode($user_search) as contents;
```

#### Result

`https://example.com/?q=hello%20world`
