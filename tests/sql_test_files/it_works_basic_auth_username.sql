select 'text' as component,
    case sqlpage.basic_auth_username()
        when 'test' then 'It works !'
        else 'error: ' || coalesce(sqlpage.basic_auth_username(), 'NULL')
    end as contents;