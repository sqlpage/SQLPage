set actual = (
    select 'first' as value
    union all
    select 'second' as value
);

select 'first' as expected, $actual as actual;

select '[{"values_json":[[1],[2]],"encoded":"a%20b"}]' as expected,
    sqlpage.run_sql('tests/sql_test_files/query_events/sqlite_json.sql') as actual;
