set scope = 'parent';
set child_rows = sqlpage.run_sql('tests/sql_test_files/variable_scope_child.sql');
select '"inherited":"parent"' as expected_contains, $child_rows as actual;
select '"changed":"child"' as expected_contains, $child_rows as actual;
select 'parent' as expected, $scope as actual;
set child_rows = sqlpage.run_sql('tests/sql_test_files/variable_scope_child.sql', '{"scope":null}');
select '"inherited":null' as expected_contains, $child_rows as actual;
select 'parent' as expected, $scope as actual;
