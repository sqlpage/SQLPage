set url = 'http://localhost:' || $echo_port || '/json';
set fetch_req = '{"method":"POST","url":"' || $url || '","body":{"hello":"world"}}';
set res = sqlpage.fetch_with_meta($fetch_req);

select '"json_body":{"hello":"world"}' as expected_contains, $res as actual;
