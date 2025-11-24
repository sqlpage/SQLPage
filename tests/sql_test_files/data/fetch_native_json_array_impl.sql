set url = 'http://localhost:' || $echo_port || '/post';
set res = sqlpage.fetch(json_object(
    'method', 'POST',
    'url', $url,
    'headers', json_object('x-custom', '1'),
    'body', json_array('hello', 'world')
));
set expected = 'POST /post|accept-encoding: br, gzip, deflate, zstd|content-length: 17|content-type: application/json|host: localhost:' || $echo_port || '|user-agent: sqlpage|x-custom: 1|["hello","world"]';
select $expected as expected,
    $res as actual;