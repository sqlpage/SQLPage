set res = sqlpage.fetch('{
    "method": "POST",
    "url": "http://localhost:62802/post",
    "headers": {"x-custom": "1"},
    "body": {"hello": "world"}
}');
set expected_like = 'POST /post
accept-encoding: br, gzip, deflate, zstd
content-length: 18
content-type: application/json
date: %
host: localhost:62802
x-custom: 1

{"hello": "world"}';
select 'text' as component,
    case
        when $res LIKE $expected_like then 'It works !'
        else 'It failed ! Expected: 
' || $expected_like || 'Got: 
' || $res
    end as contents;