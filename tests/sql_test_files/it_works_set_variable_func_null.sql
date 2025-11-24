set result = sqlpage.set_variable('x', null);
select '?' as expected, $result as actual;
