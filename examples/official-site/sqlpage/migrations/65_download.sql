-- Insert the download component into the component table
INSERT INTO
    component (name, description, icon, introduced_in_version)
VALUES
    (
        'download',
        '
The *download* component lets a page immediately return a file to the visitor.

Instead of showing a web page, it sends the file''s bytes as the whole response,
so it should be used **at the very top of your SQL page** (before the shell or any other page contents).
It is an error to use this component after another component that would display content.

How it works in simple terms:
- You provide the file content using a [data URL](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs).
A data URL is just a text string that contains both the file type and the actual data.
- Optionally, you provide a "filename" so the browser shows a proper Save As name.
If you do not provide a filename, many browsers will try to display the file inline (for example images or JSON), depending on the content type.
- You link to the page that uses the download component from another page, using the [button](/components?component=button) component for example.

What is a data URL?
- It looks like this: `data:[content-type][;base64],DATA`
- Examples:
  - Plain text (URL-encoded): `data:text/plain,Hello%20world`
  - JSON (URL-encoded): `data:application/json,%7B%22message%22%3A%22Hi%22%7D`
  - Binary data (Base64): `data:application/octet-stream;base64,SGVsbG8h`

Tips:
- Use URL encoding when you have textual data. You can use [`sqlpage.url_encode(source_text)`](/functions?function=url_encode) to encode the data.
- Use Base64 when you have binary data (images, PDFs, or content that may include special characters).
- Use [`sqlpage.read_file_as_data_url(file_path)`](/functions?function=read_file_as_data_url) to read a file from the server and return it as a data URL.

> Keep in mind that large files are better served from disk or object storage. Data URLs are best for small to medium files.
There is a big performance penalty for loading large files as data URLs, so it is not recommended.
',
        'download',
        '0.37.0'
    );

-- Insert the parameters for the download component into the parameter table
INSERT INTO
    parameter (
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
VALUES
    (
        'download',
        'data_url',
        'The file content to send, written as a data URL (for example: data:text/plain,Hello%20world or data:application/octet-stream;base64,SGVsbG8h). The part before the comma declares the content type and whether the data is base64-encoded. The part after the comma is the actual data.',
        'TEXT',
        TRUE,
        FALSE
    ),
    (
        'download',
        'filename',
        'The suggested name of the file to save (for example: report.csv). When set, the browser will download the file as an attachment with this name. When omitted, many browsers may try to display the file inline depending on its content type.',
        'TEXT',
        TRUE,
        TRUE
    );

-- Insert usage examples of the download component into the example table
INSERT INTO
    example (component, description)
VALUES
    (
        'download',
        '
## Simple plain text file
Download a small text file. The content is URL-encoded (spaces become %20).

```sql
select
    ''download'' as component,
    ''data:text/plain,Hello%20SQLPage%20world!'' as data_url,
    ''hello.txt'' as filename;
```
'
    ),
    (
        'download',
        '
## Download a PDF file from the server

Download a PDF file with the proper content type so PDF readers recognize it.
Uses [`sqlpage.read_file_as_data_url(file_path)`](/functions?function=read_file_as_data_url) to read the file from the server.

```sql
select
    ''download'' as component,
    ''report.pdf'' as filename,
    sqlpage.read_file_as_data_url(''report.pdf'') as data_url;
```
'
    ),
    (
        'download',
        '
## Serve an image stored as a BLOB in the database

### Automatically detect the mime type

If you have a table with a column `content` that contains a BLOB
(depending on the database, the type may be named `BYTEA`, `BLOB`, `VARBINARY`, or `IMAGE`),
you can just return its contents directly, and SQLPage will automatically detect the mime type,
and convert it to a data URL.

```sql
select
    ''download'' as component,
    content as data_url
from document
where id = $doc_id;
```

### Customize the mime type

In PostgreSQL, you can use the [encode(bytes, format)](https://www.postgresql.org/docs/current/functions-binarystring.html#FUNCTION-ENCODE) function to encode the file content as Base64,
and manually create your own data URL.

```sql
select
    ''download'' as component,
    ''data:'' || doc.mime_type || '';base64,'' || encode(doc.content::bytea, ''base64'') as data_url
from document as doc
where doc.id = $doc_id;
```

 - In Microsoft SQL Server, you can use the [BASE64_ENCODE(bytes)](https://learn.microsoft.com/en-us/sql/t-sql/functions/base64-encode-transact-sql) function to encode the file content as Base64.
 - In MySQL and MariaDB, you can use the [TO_BASE64(str)](https://mariadb.com/docs/server/reference/sql-functions/string-functions/to_base64) function.
'
    );