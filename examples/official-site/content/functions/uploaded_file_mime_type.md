---
icon: file-settings
since: 0.18.0
category: files-and-uploads
summary: Read the media type supplied by the browser for an uploaded file.
parameters:
  - name: name
    type: TEXT
    required: true
    description: Name of the file input field in the form.
---

Returns the MIME type of an uploaded file.

## Example: handling a picture upload

When letting the user upload a picture, you may want to check that the uploaded file is indeed an image.

```sql
select 'redirect' as component,
       'invalid_file.sql' as link
where sqlpage.uploaded_file_mime_type('myfile') not like 'image/%';
```

In `invalid_file.sql`, you can display an error message to the user:

```sql
select 'alert' as component, 'Error' as title,
    'Invalid file type' as description,
    'alert-circle' as icon, 'red' as color;
```

## Example: white-listing file types

You could have a database table containing the allowed MIME types, and check that the uploaded file is of one of those types:

```sql
select 'redirect' as component,
       'invalid_file.sql' as link
where sqlpage.uploaded_file_mime_type('myfile') not in (select mime_type from allowed_mime_types);
```
