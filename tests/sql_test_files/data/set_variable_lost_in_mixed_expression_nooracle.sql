set x = 'x';
set y = concat(1 + 1, sqlpage.request_method(), $x);
select '2GETx' as expected, $y as actual;
