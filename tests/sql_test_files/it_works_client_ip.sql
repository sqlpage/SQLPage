select 'text' as component,
    case when sqlpage.client_ip() is null then 'It works !'
        else 'It failed ! Got: ' || sqlpage.client_ip()
    end as contents;