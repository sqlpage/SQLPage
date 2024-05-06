select 'text' as component, 
    case when LENGTH(sqlpage.random_string(5)) = 5
        then 'It works !'
        else 'Error ! sqlpage.random_string(5) = ' || sqlpage.random_string(5)
    end as contents;