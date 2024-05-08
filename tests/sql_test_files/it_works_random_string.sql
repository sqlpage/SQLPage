select 'text' as component, 
    case when sqlpage.random_string(0) = '' -- with 0 as a parameter, the function becomes deterministic ;)
        then 'It works !'
        else 'Error ! sqlpage.random_string(5) = ' || sqlpage.random_string(5)
    end as contents;