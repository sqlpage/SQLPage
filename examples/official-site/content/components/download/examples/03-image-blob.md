## Serve an image stored as a BLOB in the database

### Automatically detect the mime type

If you have a table with a column `content` that contains a BLOB
(depending on the database, the type may be named `BYTEA`, `BLOB`, `VARBINARY`, or `IMAGE`),
you can just return its contents directly, and SQLPage will automatically detect the mime type,
and convert it to a data URL.

```sql
select
    'download' as component,
    content as data_url
from document
where id = $doc_id;
```

### Customize the mime type

In PostgreSQL, you can use the [encode(bytes, format)](https://www.postgresql.org/docs/current/functions-binarystring.html#FUNCTION-ENCODE) function to encode the file content as Base64,
and manually create your own data URL.

```sql
select
    'download' as component,
    'data:' || doc.mime_type || ';base64,' || encode(doc.content::bytea, 'base64') as data_url
from document as doc
where doc.id = $doc_id;
```

 - In Microsoft SQL Server, you can use the [BASE64_ENCODE(bytes)](https://learn.microsoft.com/en-us/sql/t-sql/functions/base64-encode-transact-sql) function to encode the file content as Base64.
 - In MySQL and MariaDB, you can use the [TO_BASE64(str)](https://mariadb.com/docs/server/reference/sql-functions/string-functions/to_base64) function.
