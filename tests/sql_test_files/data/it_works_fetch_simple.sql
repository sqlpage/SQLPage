set url = 'http://localhost:' || $echo_port || '/hello_world';
set res = sqlpage.fetch($url);
select 'GET /hello_world' as expected_contains,
    $res as actual;