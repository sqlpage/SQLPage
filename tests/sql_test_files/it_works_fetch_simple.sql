set url = 'http://localhost:' || $echo_port || '/hello_world';
set res = sqlpage.fetch($url);
select 'text' as component,
    case
        when $res LIKE 'GET /hello_world%' then 'It works !'
        else 'It failed ! Got: ' || $res
    end as contents;