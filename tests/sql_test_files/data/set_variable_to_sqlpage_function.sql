set my_var = sqlpage.url_encode(' ');
select '%20' as expected,
    $my_var as actual;

set encoded = (select sqlpage.url_encode(name) from (select 'a b' as name) source_rows);
select 'a%20b' as expected, $encoded as actual;
