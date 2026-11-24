---
icon: files
since: 0.49.0
category: files-and-uploads
summary: Save multiple uploads from one field and return their file details.
parameters:
  - name: files
    type: TEXT
    required: true
    description: Name of the form field containing the uploaded files. A field named `photos[]` in the form
      can be referenced as `photos`.
  - name: destination_folder
    type: TEXT
    description: |-
      Optional. Path to the folder where the files will be saved, relative to the web root. By default, the files will be saved in the `uploads` folder.

      **Security note**: this value must be a folder name you choose yourself in your SQL code (a trusted constant). Never build it from untrusted input: see [`persist_uploaded_file`](?function=persist_uploaded_file).
  - name: allowed_extensions
    type: TEXT
    description: |-
      Optional. Comma-separated list of allowed file extensions, checked for every file. By default: jpg,jpeg,png,gif,bmp,webp,pdf,txt,doc,docx,xls,xlsx,csv,mp3,mp4,wav,avi,mov.
      Changing this may be dangerous ! If you add "sql", "svg" or "html" to the list, an attacker could execute arbitrary SQL queries on your database, or impersonate other users.
  - name: mode
    type: TEXT
    description: Optional. Unix permissions to set on the files, in octal notation. By default, "600" (read/write
      for the owner only).
---

Persists all the files uploaded in a single form field, and returns a JSON array with one object per file:

 - `path`: where the file was saved, relative to the web root (like the return value of [`persist_uploaded_file`](?function=persist_uploaded_file)),
 - `name`: the original name of the file on the user's device,
 - `mime_type`: the type of the file, as sent by the browser,
 - `size`: the size of the file, in bytes.

If no file was uploaded in the field, the function returns NULL.

Use it with a file input that has the `multiple` attribute, so that users can select several files at once.
It works like [`persist_uploaded_file`](?function=persist_uploaded_file),
which only persists one file (the last one, when several were sent).

If one of the files has an extension that is not allowed, or cannot be saved, the function fails and no file is persisted.

### Example

#### Photo gallery

##### `upload_form.sql`

```sql
select 'form' as component, 'save_photos.sql' as action;
select 'file' as type, 'photos[]' as name, 'Photos' as label, true as multiple, 'image/*' as accept;
```

##### `save_photos.sql`

```sql
set files = sqlpage.persist_uploaded_files('photos', 'photos', 'jpg,jpeg,png,webp');

-- SQLite (PostgreSQL: json_array_elements($files::json), SQL Server: OPENJSON($files))
insert into photo (local_path, name, mime_type, size)
select value ->> 'path', value ->> 'name', value ->> 'mime_type', value ->> 'size'
from json_each($files);

select 'redirect' as component, 'gallery.sql' as link;
```
