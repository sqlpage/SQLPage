select 'shell' as component, 'SQLPage' as title, '/' as link;

select 'title' as component, 'SQLPage Upload Demo' as contents;

select 'card' as component, 1 as columns;
select 'Your picture' as title,
    sqlpage.read_file_as_data_url(
        sqlpage.uploaded_file_path('my_file')
    ) as top_image;

select 'debug' as component,
    sqlpage.uploaded_file_path('my_file') as uploaded_file_path,
    sqlpage.uploaded_file_mime_type('my_file') as uploaded_file_mime_type;

select 'form' as component;
select 'my_file' as name, 'file' as type, 'Picture' as label;