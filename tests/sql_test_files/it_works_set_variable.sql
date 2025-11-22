set url = sqlpage.set_variable('y', '2');
set path = sqlpage.path();
select 'text' as component,
    case
        when $url = $path || '?x=1&y=2' OR $url = $path || '?y=2&x=1' THEN 'It works !'
        else 'error: ' || $url
    end as contents;
