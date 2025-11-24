set res = sqlpage.fetch('{
  "url": "http://localhost:' || $echo_port || '/hello_world",
  "response_encoding": "base64"
}');
select 'R0VUIC9oZWxsb193b3Js' as expected_contains,
    $res as actual;
