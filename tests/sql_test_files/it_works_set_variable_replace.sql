set url = sqlpage.set_variable('x', '2');
set path = sqlpage.path();
select 'text' as component,
    case
        when $url = $path || '?x=2' THEN 'It works !'
        else 'error: ' || $url
    end as contents;
