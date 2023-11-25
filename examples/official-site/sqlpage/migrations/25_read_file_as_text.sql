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

## Example

### Rendering a markdown file

```sql
select ''text'' as component, sqlpage.read_file_as_text(''/path/to/file.md'') as text;
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
