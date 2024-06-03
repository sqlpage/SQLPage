-- display the file name of the uploaded file, unencoded, and nothing else
select 'shell-empty' as component,
    sqlpage.uploaded_file_name('my_file') as html;
