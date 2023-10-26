select 'text' as component, 
    CASE sqlpage.path()
        WHEN '/tests/sql_test_files/it_works_path.sql' THEN 'It works !'
        ELSE 'It failed ! Expected %2F, got ' || sqlpage.path() || '.'
    END
    AS contents;