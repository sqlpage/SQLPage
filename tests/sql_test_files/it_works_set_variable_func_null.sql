set url = sqlpage.set_variable('x', null);
select 'text' as component,
    case $url
        when '?' THEN 'It works !'
        else 'It failed ! Expected "?" but got ' || coalesce('"' || $url || '"', 'NULL')
    end as contents;
