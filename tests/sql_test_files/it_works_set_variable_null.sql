set path = sqlpage.path();
set url = sqlpage.set_variable('x', null);
select 'text' as component,
    case
        when $url = $path THEN 'It works !'
        else 'error: ' || $url
    end as contents;
