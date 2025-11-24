set result = sqlpage.set_variable('x', '2');
select '?x=2' as expected, $result as actual;
