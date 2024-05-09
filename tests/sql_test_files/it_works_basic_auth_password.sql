select 'text' as component,
    case sqlpage.basic_auth_password()
        when 'test' then 'It works !'
        else 'error: ' || coalesce(sqlpage.basic_auth_password(), 'NULL')
    end as contents;