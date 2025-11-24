set my_var = sqlpage.url_encode(UPPER('a'));
select 'A' as expected,
    $my_var as actual;