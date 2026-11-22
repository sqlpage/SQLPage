## Simple plain text file
Download a small text file. The content is URL-encoded (spaces become %20).

```sql
select
    'download' as component,
    'data:text/plain,Hello%20SQLPage%20world!' as data_url,
    'hello.txt' as filename;
```
