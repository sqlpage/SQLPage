select 'text' as component,
    case when sqlpage.headers() LIKE '%"cookie":"test_cook=123"%' then 'It works !'
        else 'error: ' || sqlpage.headers()
    end AS contents; 