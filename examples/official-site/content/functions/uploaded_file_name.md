---
icon: file-description
since: 0.23.0
category: files-and-uploads
summary: Read the original filename supplied by the browser.
parameters:
  - name: name
    type: TEXT
    required: true
    description: Name of the file input field in the form.
---

Returns the `filename` value in the `content-disposition` header.

## Example: saving uploaded file metadata for later download

### Making a form

```sql
select 'form' as component, 'handle_file_upload.sql' as action;
select 'myfile' as name, 'file' as type, 'File' as label;
```

### Handling the form response

### Inserting an arbitrary file as a [data URL](https://en.wikipedia.org/wiki/Data_URI_scheme) into the database

In `handle_file_upload.sql`, one can process the form results like this:

```sql
insert into uploaded_files (fname, content, uploaded) values (
  sqlpage.uploaded_file_name('myfile'),
  sqlpage.read_file_as_data_url(sqlpage.uploaded_file_path('myfile')),
  CURRENT_TIMESTAMP
);
```

> *Note*: Data URLs are larger than the original file, so it is not recommended to use them for large files.

### Downloading the uploaded files

The file can be downloaded by clicking a link like this:
```sql
select 'button' as component;
select name as title, content as link from uploaded_files where name = $file_name limit 1;
```

> *Note*: because the file is ecoded as a data uri, the file is transferred to the client whether or not the link is clicked

### Large files

See the [`sqlpage.uploaded_file_path`](?function=uploaded_file_path#function) function.

See the [`sqlpage.persist_uploaded_file`](?function=persist_uploaded_file#function) function.
