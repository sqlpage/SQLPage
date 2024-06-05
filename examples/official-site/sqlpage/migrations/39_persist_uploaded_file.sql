INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'persist_uploaded_file',
        '0.20.1',
        'device-floppy',
        'Persists an uploaded file to the local filesystem, and returns its path.
If the file input field is empty, the function returns NULL.

### Example

#### User profile picture

##### `upload_form.sql`

```sql
select ''form'' as component, ''persist_uploaded_file.sql'' as action;
select ''file'' as type, ''profile_picture'' as name, ''Upload your profile picture'' as label;
```

##### `persist_uploaded_file.sql`

```sql
update user
set profile_picture = sqlpage.persist_uploaded_file(''profile_picture'', ''profile_pictures'', ''jpg,jpeg,png,gif,webp'')
where id = (
    select user_id from session where session_id = sqlpage.cookie(''session_id'')
);
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
        'persist_uploaded_file',
        1,
        'file',
        'Name of the form field containing the uploaded file. The current page must be referenced in the `action` property of a `form` component that contains a file input field.',
        'TEXT'
    ),
    (
        'persist_uploaded_file',
        2,
        'destination_folder',
        'Optional. Path to the folder where the file will be saved, relative to the web root (the root folder of your website files). By default, the file will be saved in the `uploads` folder.',
        'TEXT'
    ),
    (
        'persist_uploaded_file',
        3,
        'allowed_extensions',
        'Optional. Comma-separated list of allowed file extensions. By default: jpg,jpeg,png,gif,bmp,webp,pdf,txt,doc,docx,xls,xlsx,csv,mp3,mp4,wav,avi,mov.
Changing this may be dangerous ! If you add "sql", "svg" or "html" to the list, an attacker could execute arbitrary SQL queries on your database, or impersonate other users.',
        'TEXT'
    );
