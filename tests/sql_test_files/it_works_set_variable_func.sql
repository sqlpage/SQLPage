set url = sqlpage.set_variable('y', '2');

select 'text' as component,
    case $url
        when '?x=1&y=2' THEN 'It works !'
        else 'It failed ! Expected ?x=1&y=2 but got ' || coalesce($url, 'NULL')
    end as contents;