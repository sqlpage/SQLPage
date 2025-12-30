set x = '2';
select '{"x":"2"}' as expected, sqlpage.variables() as actual;
