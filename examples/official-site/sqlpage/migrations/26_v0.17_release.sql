INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES (
        'SQLPage v0.17',
        'SQLPage v0.17 introduces file uploads, HTTPS, and more.',
        'git-fork',
        '2023-11-28',
        '
# SQLPage v0.17 is out !

[SQLPage](/) is a web application server that lets you build entire web applications with just SQL queries.
v0.17 was just released, and it''s worth a blog post to highlight some of the coolest new features.

Mostly, this release makes it a matter of minutes to build a data import pipeline for your website,
and a matter of seconds to deploy your SQLPage website securely with automatic HTTPS certificates.

## Uploads

This release is all about a long awaited feature: **file uploads**.
Your SQLPage website can now accept file uploads from users,
store them either in a directory or directly in a database table.

You can add a file upload button to a form with a simple 

```sql
select ''form'' as component;
select ''profile_picture'' as name, ''file'' as type;
```

when received by the server, the file will be saved in a temporary directory
(customizable with `TMPDIR` on linux).
You can access the temporary file path with
the new [`sqlpage.uploaded_file_path`](/functions.sql?function=uploaded_file_path#function) function.

You can then persist the upload as a permanent file on the server with the
[`sqlpage.exec`](https://sql-page.com/functions.sql?function=exec#function) function:

```sql
set file_path = sqlpage.uploaded_file_path(''profile_picture'');
select sqlpage.exec(''mv'', $file_path, ''/path/to/my/file'');
```

or you can store it directly in a database table with the new
[`sqlpage.read_file_as_data_url`](https://sql-page.com/functions.sql?function=read_file_as_data_url#function) and
[`sqlpage.read_file_as_text`](https://sql-page.com/functions.sql?function=read_file_as_text#function) functions:

```sql
insert into files (url) values (sqlpage.read_file_as_data_url(sqlpage.uploaded_file_path(''profile_picture'')))
returning ''text'' as component, ''Uploaded new file with id: '' || id as contents;
```

The maximum size of uploaded files is configurable with the [`max_uploaded_file_size`](https://github.com/sqlpage/SQLPage/blob/main/configuration.md)
configuration parameter. By default, it is set to 5 MiB.

### Parsing CSV files

SQLPage can also parse uploaded [CSV](https://en.wikipedia.org/wiki/Comma-separated_values) files and insert them directly into a database table.
SQLPage re-uses PostgreSQL''s [`COPY` syntax](https://www.postgresql.org/docs/current/sql-copy.html)
to import the CSV file into the database.
When connected to a PostgreSQL database, SQLPage will use the native `COPY` statement,
for super fast and efficient on-database CSV parsing.
But it will also work with any other database as well, by
parsing the CSV locally and emulating the same behavior with simple `INSERT` statements.

#### `user_file_upload.sql`
```sql
select ''form'' as component, ''bulk_user_import.sql'' as action;
select ''user_csv_file'' as name, ''file'' as type, ''text/csv'' as accept;
```

#### `bulk_user_import.sql`
```sql
-- create a temporary table to preprocess the data
create temporary table if not exists csv_import(name text, age text);
delete from csv_import; -- empty the table
-- If you don''t have any preprocessing to do, you can skip the temporary table and use the target table directly

copy csv_import(name, age) from ''user_csv_file''
with (header true, delimiter '','', quote ''"'', null ''NaN''); -- all the options are optional
-- since header is true, the first line of the file will be used to find the "name" and "age" columns
-- if you don''t have a header line, the first column in the CSV will be interpreted as the first column of the table, etc

-- run any preprocessing you want on the data here

-- insert the data into the users table
insert into users (name, birth_date)
select upper(name), date_part(''year'', CURRENT_DATE) - cast(age as int) from csv_import;
```

### New functions

#### Handle uploaded files

 - [`sqlpage.uploaded_file_path`](https://sql-page.com/functions.sql?function=uploaded_file_path#function) to get the temprary local path of a file uploaded by the user. This path will be valid until the end of the current request, and will be located in a temporary directory (customizable with `TMPDIR`). You can use [`sqlpage.exec`](https://sql-page.com/functions.sql?function=exec#function) to operate on the file, for instance to move it to a permanent location.
 - [`sqlpage.uploaded_file_mime_type`](https://sql-page.com/functions.sql?function=uploaded_file_mime_type#function) to get the type of file uploaded by the user. This is the MIME type of the file, such as `image/png` or `text/csv`. You can use this to easily check that the file is of the expected type before storing it.

 The new [*Image gallery* example](https://github.com/sqlpage/SQLPage/tree/main/examples/image%20gallery%20with%20user%20uploads)
in the official repository shows how to use these functions to create a simple image gallery with user uploads.

#### Read files

These new functions are useful to read the contents of a file uploaded by the user,
but can also be used to read any file on the computer where SQLPage is running:

 - [`sqlpage.read_file_as_text`](https://sql-page.com/functions.sql?function=read_file_as_text#function) reads the contents of a file on the server and returns a text string.
 - [`sqlpage.read_file_as_data_url`](https://sql-page.com/functions.sql?function=read_file_as_data_url#function) reads the contents of a file on the server and returns a [data URL](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs). This is useful to embed images directly in web pages, or make link

## HTTPS

This is the other big feature of this release: SQLPage now supports HTTPS !
Until now, if you wanted to use HTTPS with SQLPage, you had to put it behind a
*reverse proxy*, which is what the official documentation website does.

This required a lot of manual configuration
that would compromise your security if you get it wrong.

With SQLPage v0.17, you just give your domain name,
and it takes care of everything.

And while we''re at it, SQLPage also supports HTTP/2, for even faster page loads.

To enable HTTPS, you need to buy a [domain name](https://en.wikipedia.org/wiki/Domain_name)
and make it point to the server where SQLPage is running.
Then set the `https_domain` configuration parameter to `yourdomain.com` in your [`sqlpage.json` configuration file](./configuration.md).

```json
{
  "https_domain": "my-cool-website.com"
}
```

That''s it. No external tool to install, no certificate to generate, no configuration to tweak.
No need to restart SQLPage regularly either, or to worry about renewing your certificate when it expires.
SQLPage will automatically request a certificate from [Let''s Encrypt](https://letsencrypt.org/) by default,
and does not even need to listen on port 80 to do so.

## SQL parser improvements

SQLPage needs to parse SQL queries to be able to bind the right parameters to them,
and to inject the results of built-in sqlpage functions in them.
The parser we use is very powerful and supports most SQL features,
but there are some edge cases where it fails to parse a query.
That''s why we contribute to it a lot, and bring the latest version of the parser to SQLPage as soon as it is released.

### JSON functions in MS SQL Server

SQLPage now supports the [`FOR JSON` syntax](https://learn.microsoft.com/en-us/sql/relational-databases/json/format-query-results-as-json-with-for-json-sql-server?view=sql-server-ver16&tabs=json-path) in MS SQL Server.

This unlocks a lot of new possibilities, that were previously only available in other databases.

This is particularly interesting to build complex menus with the `shell` component,
to build multiple-answer select inputs with the `form` component,
and to create JSON APIs.

### Other sql syntax enhancements

 - SQLPage now supports the custom `CONVERT` expression syntax for MS SQL Server, and the one for MySQL.
 - The `VARCHAR(MAX)` type in MS SQL Server new works. We now use it for all variables bound as parameters to your SQL queries (we used to use `VARCHAR(8000)` before).
 - `INSERT INTO ... DEFAULT VALUES ...` is now parsed correctly.

## Other news

 - Dates and timestamps returned from the database are now always formatted in ISO 8601 format, which is the standard format for dates in JSON. This makes it easier to use dates in SQLPage.
 - The `cookie` component now supports setting an explicit expiration date for cookies.
 - The `cookie` component now supports setting the `SameSite` attribute of cookies, and defaults to `SameSite=Strict` for all cookies. What this means in practice is that cookies set by SQLPage will not be sent to your website if the user is coming from another website. This prevents someone from tricking your users into executing SQLPage queries on your website by sending them a malicious link.
 - Bugfix: setting `min` or `max` to `0` in a number field in the `form` component now works as expected.
 - Added support for `.env` files to set SQLPage''s [environment variables](./configuration.md#environment-variables).
 - Better responsive design in the card component. Up to 5 cards per line on large screens. The number of cards per line is still customizable with the `columns` attribute.
 - [New icons](https://tabler-icons.io/changelog): 
   - ![new icons in tabler 42](https://github.com/tabler/tabler-icons/assets/1282324/00856af9-841d-4aa9-995d-121c7ddcc005)
');