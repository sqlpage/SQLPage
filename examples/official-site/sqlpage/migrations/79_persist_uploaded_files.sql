INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'persist_uploaded_files',
        '0.49.0',
        'files',
        'Persists all the files uploaded in a single form field, and returns their paths as a JSON array.
If no file was uploaded in the field, the function returns NULL.

Use it with a file input that has the `multiple` attribute, so that users can select several files at once.
It works like [`persist_uploaded_file`](?function=persist_uploaded_file),
which only persists one file (the last one, when several were sent).

If one of the files has an extension that is not allowed, the function fails and no file is persisted.

### Example

#### Photo gallery

##### `upload_form.sql`

```sql
select ''form'' as component, ''save_photos.sql'' as action;
select ''file'' as type, ''photos[]'' as name, ''Photos'' as label, true as multiple, ''image/*'' as accept;
```

##### `save_photos.sql`

```sql
set paths = sqlpage.persist_uploaded_files(''photos'', ''photos'', ''jpg,jpeg,png,webp'');

insert into photo (path)
select value from json_each($paths); -- PostgreSQL: json_array_elements_text($paths::json)

select ''redirect'' as component, ''gallery.sql'' as link;
```
'
    );
INSERT INTO sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES (
        'persist_uploaded_files',
        1,
        'files',
        'Name of the form field containing the uploaded files. A field named `photos[]` in the form can be referenced as `photos`.',
        'TEXT'
    ),
    (
        'persist_uploaded_files',
        2,
        'destination_folder',
        'Optional. Path to the folder where the files will be saved, relative to the web root. By default, the files will be saved in the `uploads` folder.

**Security note**: this value must be a folder name you choose yourself in your SQL code (a trusted constant). Never build it from untrusted input: see [`persist_uploaded_file`](?function=persist_uploaded_file).',
        'TEXT'
    ),
    (
        'persist_uploaded_files',
        3,
        'allowed_extensions',
        'Optional. Comma-separated list of allowed file extensions, checked for every file. By default: jpg,jpeg,png,gif,bmp,webp,pdf,txt,doc,docx,xls,xlsx,csv,mp3,mp4,wav,avi,mov.
Changing this may be dangerous ! If you add "sql", "svg" or "html" to the list, an attacker could execute arbitrary SQL queries on your database, or impersonate other users.',
        'TEXT'
    ),
    (
        'persist_uploaded_files',
        4,
        'mode',
        'Optional. Unix permissions to set on the files, in octal notation. By default, "600" (read/write for the owner only).',
        'TEXT'
    );
