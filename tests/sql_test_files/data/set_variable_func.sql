set result = sqlpage.set_variable('y', '2');
select '?x=1&y=2' as expected, $result as actual;
