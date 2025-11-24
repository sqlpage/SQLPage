select '/tests/sql_test_files/data/it_works_path.sql' as expected,
    sqlpage.path() as actual;