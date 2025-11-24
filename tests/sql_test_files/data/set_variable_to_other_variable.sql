SET x = 42;
SET y = $x;
SET z = $y;
select '42' as expected, $z as actual;
