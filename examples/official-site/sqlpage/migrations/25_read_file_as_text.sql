-- Insert the 'variables' function into sqlpage_functions table
INSERT INTO sqlpage_functions (
    "name",
    "introduced_in_version",
    "icon",
    "description_md"
)
VALUES (
    'read_file_as_text',
    '0.17.0',
    'file-invoice',
    'Returns a string containing the contents of the given file.

The file must be a raw text file using UTF-8 encoding.

The file path is relative to the `web root` directory, which is the directory from which your website is served
(not necessarily the directory SQLPage is launched from).

If the given argument is null, the function will return null.

As with other functions, if an error occurs during execution 
(because the file does not exist, for instance),
the function will display an error message and the
database query will not be executed.

If you are using a `sqlpage_files` table to store files directly in the database (serverless mode),
the function will attempt to read the file from the database filesystem if it is not found on the local disk,
using the same logic as for serving files in response to HTTP requests.

## Example

### Rendering a markdown file

```sql
select ''text'' as component, sqlpage.read_file_as_text(''/path/to/file.md'') as contents_md;
```
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
    'read_file_as_text',
    1,
    'name',
    'Path to the file to read.',
    'TEXT'
);
