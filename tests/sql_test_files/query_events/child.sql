select sqlpage.run_sql('tests/sql_test_files/query_events/grandchild.sql') as nested from (select 1 as n) source_rows;
