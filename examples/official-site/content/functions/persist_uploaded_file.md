---
icon: device-floppy
since: 0.20.1
category: files-and-uploads
summary: Save an uploaded file and return its path.
parameters:
  - name: file
    type: TEXT
    required: true
    description: Name of the form field containing the uploaded file. The current page must be referenced in
      the `action` property of a `form` component that contains a file input field.
  - name: destination_folder
    type: TEXT
    description: |-
      Optional. Path to the folder where the file will be saved, relative to the web root (the root folder of your website files). By default, the file will be saved in the `uploads` folder.

      **Security note**: this value must be a folder name you choose yourself in your SQL code (a trusted constant). Never build it from untrusted input such as a form field, a query parameter, a request header, or anything else the visitor controls. The folder is joined directly to the web root, so a value containing `..` or an absolute path would cause the uploaded file to be written *outside* the web root, anywhere the SQLPage process can write. Keeping `destination_folder` a fixed value chosen by the application author avoids this.
  - name: allowed_extensions
    type: TEXT
    description: |-
      Optional. Comma-separated list of allowed file extensions. By default: jpg,jpeg,png,gif,bmp,webp,pdf,txt,doc,docx,xls,xlsx,csv,mp3,mp4,wav,avi,mov.
      Changing this may be dangerous ! If you add "sql", "svg" or "html" to the list, an attacker could execute arbitrary SQL queries on your database, or impersonate other users.
  - name: mode
    type: TEXT
    description: |-
      Optional. Unix permissions to set on the file, in octal notation. By default, the file will be saved with "600" (read/write for the owner only).
      Octal notation works by using three digits from 0 to 7: the first for the owner, the second for the group, and the third for others.
      For example, "644" means read/write for the owner, and read-only for others.
      [Learn more about numeric notation for file-system permissions on Wikipedia](https://en.wikipedia.org/wiki/File-system_permissions#Numeric_notation).
---

Persists an uploaded file to the local filesystem, and returns its path.
If the file input field is empty, the function returns NULL.

### Example

#### User profile picture

##### `upload_form.sql`

```sql
select 'form' as component, 'persist_uploaded_file.sql' as action;
select 'file' as type, 'profile_picture' as name, 'Upload your profile picture' as label;
```

##### `persist_uploaded_file.sql`

```sql
update user
set profile_picture = sqlpage.persist_uploaded_file('profile_picture', 'profile_pictures', 'jpg,jpeg,png,gif,webp')
where id = (
    select user_id from session where session_id = sqlpage.cookie('session_id')
);
```
