set x = 'set_value';
set set_only = 'only_in_set';

select 'set_value' as expected, $x as actual;
select 'only_in_set' as expected, $set_only as actual;
select '{"x":"1"}' as expected, sqlpage.variables('get') as actual;
select '"x":"set_value"' as expected_contains, sqlpage.variables('set') as actual;
select '"set_only":"only_in_set"' as expected_contains, sqlpage.variables('set') as actual;
select '"x":"set_value"' as expected_contains, sqlpage.variables() as actual;
select '"set_only":"only_in_set"' as expected_contains, sqlpage.variables() as actual;

-- The fixture runner provides ?x=1. SET NULL suppresses $x but leaves GET enumeration immutable.
set x = NULL;
select NULL as expected, $x as actual;
select '"x":null' as expected_contains, sqlpage.variables() as actual;
select '"x":"1"' as expected_contains, sqlpage.variables('get') as actual;
