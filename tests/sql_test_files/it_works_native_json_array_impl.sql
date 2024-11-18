set res = sqlpage.fetch(json_object(
    'method', 'POST',
    'url', 'http://localhost:62802/post',
    'headers', json_object('x-custom', '1'),
    'body', json_array('hello', 'world')
));
set expected = 'POST /post|accept-encoding: br, gzip, deflate, zstd|content-length: 17|content-type: application/json|host: localhost:62802|user-agent: sqlpage|x-custom: 1|["hello","world"]';
select 'text' as component,
    case $res
        when $expected then 'It works !'
        else 'It failed ! Expected: 
' || $expected || '
Got: 
' || $res
    end as contents;