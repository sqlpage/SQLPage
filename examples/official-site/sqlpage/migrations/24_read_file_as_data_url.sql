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

## Example: inlining a picture
    
```sql
select ''card'' as component;
select ''Picture'' as title, sqlpage.read_file_as_data_url(''/path/to/picture.jpg'') as top_image;
```

> **Note:** Data URLs are larger than the original file they represent, so they should only be used for small files (a few kilobytes).
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
