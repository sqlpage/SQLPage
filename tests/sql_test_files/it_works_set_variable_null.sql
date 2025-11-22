set url = sqlpage.set_variable('x', null);
set path = sqlpage.path();
select 'text' as component,
    case
        when $url = $path THEN 'It works !'
        else 'error: ' || $url
    end as contents;
