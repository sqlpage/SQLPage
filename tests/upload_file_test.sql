select 'text' as component,
    COALESCE(
        sqlpage.read_file_as_text(sqlpage.uploaded_file_path('my_file')),
        'No file uploaded'
     ) as contents;