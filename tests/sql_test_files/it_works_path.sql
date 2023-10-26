select 'text' as component, 
    CASE sqlpage.path()
        WHEN '/tests/sql_test_files/it_works_path.sql' THEN 'It works !'
        ELSE 'It failed ! Expected "/tests/sql_test_files/it_works_path.sql", got "' || sqlpage.path() || '"".'
    END
    AS contents;