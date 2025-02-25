set res = sqlpage.fetch_with_meta('http://not-a-real-url');

select 'text' as component,
    case
        when $res LIKE '%"error":"Request failed%' then 'It works !'
        else CONCAT('Error! Got: ', $res)
    end as contents; 