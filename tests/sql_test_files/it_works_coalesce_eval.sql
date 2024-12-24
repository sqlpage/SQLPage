select 'text' as component, 
    case sqlpage.link(coalesce($i_do_not_exist, 'https://example.com'))
        when 'https://example.com' then 'It works !'
        else 'error: ' || coalesce(sqlpage.link(coalesce($i_do_not_exist, 'https://example.com')), 'NULL')
    end AS contents;