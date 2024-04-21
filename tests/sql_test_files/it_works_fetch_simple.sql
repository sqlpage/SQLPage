set res = sqlpage.fetch('http://localhost:62802/hello_world')
select 'text' as component,
    case
        when $res LIKE 'GET /hello_world%' then 'It works !'
        else 'It failed ! Got: ' || $res
    end as contents;