-- The fixture runner provides ?x=1. SET NULL suppresses $x but leaves GET enumeration immutable.
set x = NULL;
select NULL as expected, $x as actual;
select '"x":null' as expected_contains, sqlpage.variables() as actual;
select '"x":"1"' as expected_contains, sqlpage.variables('get') as actual;
