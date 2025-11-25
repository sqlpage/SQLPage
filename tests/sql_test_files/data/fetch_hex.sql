set res = sqlpage.fetch('{
  "url": "http://localhost:' || $echo_port || '/hello_world",
  "response_encoding": "hex"
}');
select '474554202f68656c6c6f5f776f726c64' as expected_contains,
    $res as actual;
