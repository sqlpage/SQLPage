set res = sqlpage.fetch('{
    "method": "POST",
    "url": "http://localhost:62802/post",
    "headers": {"x-custom": "1"},
    "body": {"hello": "world"}
}');
set expected = 'POST /post|accept-encoding: br, gzip, deflate, zstd|content-length: 18|content-type: application/json|host: localhost:62802|x-custom: 1|{"hello": "world"}';
select 'text' as component,
    case $res
        when $expected then 'It works !'
        else 'It failed ! Expected: 
' || $expected || '
Got: 
' || $res
    end as contents;