select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'title' as component, 'SQLPage Image Upload Demo' as contents;

set $data_url = sqlpage.read_file_as_data_url(sqlpage.uploaded_file_path('my_file'));

select 'card' as component, 1 as columns where $data_url is not null;
select 'Your picture' as title,
    $data_url as top_image,
    'Uploaded file type: ' || sqlpage.uploaded_file_mime_type('my_file') as description
where $data_url is not null;

select 'form' as component;
select 'my_file' as name, 'file' as type, 'Picture' as label;

select 'text' as component, '
## About

This is a demo of the SQLPage file upload feature.
A file upload form is created using the [form](/documentation.sql?component=form#component) component: 

```sql
select ''form'' as component;
select ''my_file'' as name, ''file'' as type, ''Picture'' as label;
```

When a file is uploaded, it is displayed in a [card](/documentation.sql?component=card#component) component
using the [sqlpage.read_file_as_data_url](/functions.sql?function=read_file_as_data_url#function) function:

```sql
select ''card'' as component, 1 as columns;
select ''Your picture'' as title, sqlpage.read_file_as_data_url(sqlpage.uploaded_file_path(''my_file'')) as top_image;
```

[See the source code of this page](https://github.com/lovasoa/SQLpage/blob/main/examples/official-site/examples/handle_picture_upload.sql).
' as contents_md;