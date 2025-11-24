set my_var = sqlpage.url_encode(' ');
select '%20' as expected,
    $my_var as actual;