-- Checks that the UPPER function is working correctly with unicode characters.
select 'text' as component,
    case
        when UPPER('é') = 'É' then 'It works !'
        else 'It failed ! Expected É, got ' || UPPER('é') || '.'
    end as contents;