set url = 'http://localhost:' || $echo_port || '/post';
set fetch_request = '{"method": "POST", "url": "' || $url || '", "headers": {"x-custom": "1"}, "body": {"hello": "world"}}';
set res = sqlpage.fetch($fetch_request);
set expected = 'POST /post|accept-encoding: br, gzip, deflate, zstd|content-length: 18|content-type: application/json|host: localhost:' || $echo_port || '|user-agent: sqlpage|x-custom: 1|{"hello": "world"}';
select $expected as expected,
    $res as actual;