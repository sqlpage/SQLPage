set actual = sqlpage.uploaded_file_path('my_file');
select 'NULL' as expected,
    coalesce($actual, 'NULL') as actual;
