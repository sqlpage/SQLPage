set url = 'http://localhost:' || $echo_port || '/hello_world';
set fetch_req = '{
    "method": "PUT",
    "url": "' || $url || '",
    "headers": {
        "user-agent": "myself"
    }
}';
set res = sqlpage.fetch_with_meta($fetch_req);

select 'text' as component,
    case
        when $res LIKE '%"status":200%' AND $res LIKE '%"headers":{%' AND $res LIKE '%"body":"%' then 'It works !'
        else 'Error! Got: ' || $res
    end as contents; 