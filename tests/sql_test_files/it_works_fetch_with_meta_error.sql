set res = sqlpage.fetch_with_meta('http://not-a-real-url');

select 'text' as component,
    case
        when json_extract($res, '$.error') LIKE 'Request failed%' then 'It works !'
        else 'Error! Got: ' || $res
    end as contents; 