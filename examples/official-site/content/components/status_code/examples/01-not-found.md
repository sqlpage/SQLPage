Set the HTTP status code to 404, indicating that the requested resource was not found.
Useful in combination with [`404.sql` files](/your-first-sql-website/custom_urls.sql):

```sql
SELECT 'status_code' as component, 404 as status;
```
