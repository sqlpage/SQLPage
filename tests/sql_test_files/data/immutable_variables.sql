set x = 'set_value';
set set_only = 'only_in_set';

select 'set_value' as expected, $x as actual;
select 'only_in_set' as expected, $set_only as actual;
select '{"x":"1"}' as expected, sqlpage.variables('get') as actual;
select '"x":"set_value"' as expected_contains, sqlpage.variables('set') as actual;
select '"set_only":"only_in_set"' as expected_contains, sqlpage.variables('set') as actual;
select '"x":"set_value"' as expected_contains, sqlpage.variables() as actual;
select '"set_only":"only_in_set"' as expected_contains, sqlpage.variables() as actual;
