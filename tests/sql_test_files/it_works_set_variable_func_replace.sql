set url = sqlpage.set_variable('x', '2');
select 'text' as component,
    case $url
        when '?x=2' THEN 'It works !'
        else 'It failed ! Expected ?x=2 but got ' || $url
    end as contents;
