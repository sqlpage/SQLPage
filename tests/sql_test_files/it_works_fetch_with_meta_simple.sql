set res = sqlpage.fetch_with_meta('{
    "method": "PUT",
    "url": "http://localhost:62802/hello_world",
    "headers": {
        "user-agent": "myself"
    }
}');

select 'text' as component,
    case
        when $res LIKE '%"status":200%' AND $res LIKE '%"headers":{%' AND $res LIKE '%"body":"%' then 'It works !'
        else 'Error! Got: ' || $res
    end as contents; 