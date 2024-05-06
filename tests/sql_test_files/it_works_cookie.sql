select 'text' as component, 
    case sqlpage.cookie('test_cook')
        when '123' then 'It works !'
        else 'error: ' || coalesce(sqlpage.cookie('test_cook'), 'NULL')
    end AS contents;