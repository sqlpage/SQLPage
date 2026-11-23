---
rows:
  - component: form
    enctype: multipart/form-data
    title: Upload a picture
    validate: Upload
    action: examples/handle_picture_upload.sql
  - name: my_file
    type: file
    accept: image/png, image/jpeg
    label: Picture
    description: Upload a small picture
    required: true
---

## File upload

You can use the `file` type to allow the user to upload a file.

The file will be uploaded to the server, and you will be able to access it using the
[`sqlpage.uploaded_file_path`](functions.sql?function=uploaded_file_path#function) function.

Here is how you could save the uploaded file to a table in the database:

```sql
INSERT INTO uploaded_file(name, data)
VALUES (
    :filename,
    sqlpage.read_file_as_data_url(sqlpage.uploaded_file_path('my_file'))
)
```
