-- display the data url of the uploaded file, unencoded, and nothing else
select 'shell-empty' as component,
    sqlpage.read_file_as_data_url(sqlpage.uploaded_file_path('my_file')) as html;