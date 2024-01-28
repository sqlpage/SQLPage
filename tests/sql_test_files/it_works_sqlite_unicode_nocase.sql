-- Checks that the NOCASE collation is working correctly with unicode characters.
select 'text' as component,
    case
        when 'é' = 'É' COLLATE NOCASE then 'It works !'
        else 'It failed ! Expected "é" = "É" COLLATE NOCASE.'
    end as contents;