INSERT INTO
        sqlpage_functions (
                "name",
                "introduced_in_version",
                "icon",
                "description_md"
        )
VALUES
        (
                'client_ip',
                '0.33.0',
                'network',
                'Returns the IP address of the client making the HTTP request.

### Example

```sql
insert into connection_log (client_ip) values (sqlpage.client_ip());
```

### Details

The function returns:
- The IP address of the client as a string
- `null` if the client IP cannot be determined (e.g., when serving through a Unix socket)

### ⚠️ Important Notes for Production Use

When [running behind a reverse proxy](/your-first-sql-website/nginx.sql) (e.g., Nginx, Apache, Cloudflare):
- This function will return the IP address of the reverse proxy, not the actual client
- To get the real client IP, use [`sqlpage.header`](?function=header): `sqlpage.header(''x-forwarded-for'')` or `sqlpage.header(''x-real-ip'')`
  - The exact header name depends on your reverse proxy configuration

Example with reverse proxy:
```sql
-- Choose the appropriate header based on your setup
select coalesce(
    sqlpage.header(''x-forwarded-for''),
    sqlpage.header(''x-real-ip''),
    sqlpage.client_ip()
) as real_client_ip;
```

For security-critical applications, ensure your reverse proxy is properly configured to set and validate these headers.
'
        );