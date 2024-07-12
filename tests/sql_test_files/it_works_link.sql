select 'text' as component, 
    case sqlpage.link('test.sql', json_object('x', 123))
        when 'test.sql?x=123' then 'It works !'
        else 'error: ' || coalesce(sqlpage.link('test.sql', json_object('x', 123)), 'NULL')
    end AS contents;