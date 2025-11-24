set url = 'http://localhost:' || $echo_port || '/hello_world';
set fetch_req = '{
    "method": "PUT",
    "url": "' || $url || '",
    "headers": {
        "user-agent": "myself"
    }
}';
set res = sqlpage.fetch_with_meta($fetch_req);

select '"status":200' as expected_contains,
       '"headers":{' as expected_contains,
       '"body":"' as expected_contains,
       $res as actual;
