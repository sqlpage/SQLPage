-- Insert the 'variables' function into sqlpage_functions table
INSERT INTO sqlpage_functions (
    "name",
    "introduced_in_version",
    "icon",
    "description_md"
)
VALUES (
    'read_file_as_data_url',
    '0.17.0',
    'file-dollar',
    'Returns a [data URL](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs)
containing the contents of the given file.

The file path is relative to the `web root` directory, which is the directory from which your website is served.
By default, this is the directory SQLPage is launched from, but you can change it
with the `web_root` [configuration option](https://github.com/sqlpage/SQLPage/blob/main/configuration.md).

If the given argument is null, the function will return null.

As with other functions, if an error occurs during execution 
(because the file does not exist, for instance),
the function will display an error message and the
database query will not be executed.

If you are using a `sqlpage_files` table to store files directly in the database (serverless mode),
the function will attempt to read the file from the database filesystem if it is not found on the local disk,
using the same logic as for serving files in response to HTTP requests.

## MIME type

Data URLs contain the [MIME type](https://en.wikipedia.org/wiki/Media_type) of the file they represent.
If the first argument to this function is the result of a call to the `sqlpage.uploaded_file_path` function,
the declared MIME type of the uploaded file transmitted by the browser will be used.

Otherwise, the MIME type will be guessed from the file extension, without looking at the file contents.


## Example: inlining a picture
    
```sql
select ''card'' as component;
select ''Picture'' as title, sqlpage.read_file_as_data_url(''/path/to/picture.jpg'') as top_image;
```

> **Note:** Data URLs are larger than the original file they represent, so they should only be used for small files
> (under a few hundred kilobytes).
> Otherwise, the page will take a long time to load.
');

-- Insert the parameters for the 'variables' function into sqlpage_function_parameters table
-- Parameter 1: 'method' parameter
INSERT INTO sqlpage_function_parameters (
    "function",
    "index",
    "name",
    "description_md",
    "type"
)
VALUES (
    'read_file_as_data_url',
    1,
    'name',
    'Path to the file to read.',
    'TEXT'
);
