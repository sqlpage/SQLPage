select 'text' as component, 
    case sqlpage.header('cookie')
        when 'test_cook=123' then 'It works !'
        else 'error: ' || coalesce(sqlpage.header('cookie'), 'NULL')
    end AS contents;