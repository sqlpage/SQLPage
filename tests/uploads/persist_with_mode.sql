set contents = sqlpage.persist_uploaded_file('my_file', 'tests_uploads', 'txt', $mode);
select 'text' as component, $contents as contents;
