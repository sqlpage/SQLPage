select 'text' as component,
    sqlpage.read_file_as_text(sqlpage.uploaded_file_path('my_file')) as contents;