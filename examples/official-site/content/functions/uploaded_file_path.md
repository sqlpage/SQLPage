---
icon: upload
since: 0.17.0
category: files-and-uploads
summary: Get the temporary path of an uploaded file.
parameters:
  - name: name
    type: TEXT
    required: true
    description: Name of the file input field in the form.
---

Returns the path to a temporary file containing the contents of an uploaded file.

## Example: handling a picture upload

### Making a form

```sql
select 'form' as component, 'handle_picture_upload.sql' as action;
select 'myfile' as name, 'file' as type, 'Picture' as label;
select 'title' as name, 'text' as type, 'Title' as label;
```

### Handling the form response

### Inserting an image file as a [data URL](https://en.wikipedia.org/wiki/Data_URI_scheme) into the database

In `handle_picture_upload.sql`, one can process the form results like this:

```sql
insert into pictures (title, path) values (:title, sqlpage.read_file_as_data_url(sqlpage.uploaded_file_path('myfile')));
```

> *Note*: Data URLs are larger than the original file, so it is not recommended to use them for large files.

### Inserting file contents as text into the database

When the uploaded file is a simple raw text file (e.g. a `.txt` file),
one can use the [`sqlpage.read_file_as_text`](?function=read_file_as_text#function)
function to insert the contents of the file into the database like this:

```sql
insert into text_documents (title, path) values (:title, sqlpage.read_file_as_text(sqlpage.uploaded_file_path('my_text_file')));
```

### Saving the uploaded file to a permanent location

When the uploaded file is larger than a few megabytes, it is not recommended to store it in the database.
Instead, one can save the file to a permanent location on the server, and store the path to the file in the database.

You can move the file to a permanent location using the [`sqlpage.persist_uploaded_file`](?function=persist_uploaded_file#function) function.
### Advanced file handling

For more advanced file handling, such as uploading files to a cloud storage service,
you can write a small script in your favorite programming language,
and call it using the [`sqlpage.exec`](?function=exec#function) function.

For instance, one could save the following small bash script to `/usr/local/bin/upload_to_s3`:

```bash
#!/bin/bash
aws s3 cp "$1" s3://your-s3-bucket-name/
echo "https://your-s3-bucket-url/$(basename "$1")"
```

Then, you can call it from SQL like this:

```sql
set url = sqlpage.exec('upload_to_s3', sqlpage.uploaded_file_path('myfile'));
insert into uploaded_files (title, path) values (:title, $url);
```
