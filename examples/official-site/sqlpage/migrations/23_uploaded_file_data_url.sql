-- Insert the 'variables' function into sqlpage_functions table
INSERT INTO sqlpage_functions (
    "name",
    "introduced_in_version",
    "icon",
    "description_md"
)
VALUES (
    'uploaded_file_data_url',
    '0.17.0',
    'file-invoice',
    'Returns a [data URL](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs)
containing the contents of an uploaded file.

## Example: handling a picture upload

### Making a form

```sql
select ''form'' as component, ''handle_picture_upload.sql'' as action;
select ''myfile'' as name, ''file'' as type, ''Picture'' as label;
select ''title'' as name, ''text'' as type, ''Title'' as label;
```

### Handling the form response

In `handle_picture_upload.sql`, one can process the form results like this:

```sql
insert into pictures (title, data_url) values (:title, sqlpage.uploaded_file_data_url(''myfile''));
```

The the picture can be displayed like this:

```sql
select ''card'' as component;
select title, data_url as top_image from pictures;
```

You can also make simple self-contained links to data URLs like this:

```sql
select ''list'' as component;
select title, data_url as link from pictures;
```
'
);

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
    'uploaded_file_data_url',
    1,
    'name',
    'Name of the file input field.',
    'TEXT'
);
