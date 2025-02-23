set res = sqlpage.fetch_with_meta('{
    "method": "PUT",
    "url": "http://localhost:62802/hello_world",
    "headers": {
        "user-agent": "myself"
    }
}');
select 'text' as component,
    case
        when json_extract($res, '$.status') = 200
        and cast(json_extract($res, '$.headers.content-length') as int) > 100
        and json_extract($res, '$.body') like 'PUT /hello_world%'
        then 'It works !'
        else 'It failed! Got: ' || $res
    end as contents; 