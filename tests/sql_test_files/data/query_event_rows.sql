-- Compare the complete rows, including order and absence of private input columns.
set included = json_array(json_object('nested', '[{"value":"nested"}]'));
set expected = json_array(
    json_object('n', 'row1', 'included', $included),
    json_object('n', 'row2', 'included', $included),
    json_object('scalar_included', $included),
    json_object('values_json', json_array('[1]', '[2]'), 'encoded', json_array('a b', 'a%20b'))
);
select $expected as expected, sqlpage.run_sql('tests/sql_test_files/query_events/rows.sql') as actual;
