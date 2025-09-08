select 'text' as component, 
    case sqlpage.link('test.sql', json_object('x', null))
        when 'test.sql' then 'It works !'
        else 'error: ' || coalesce(sqlpage.link('test.sql', json_object('x', null)), 'NULL')
    end AS contents;
