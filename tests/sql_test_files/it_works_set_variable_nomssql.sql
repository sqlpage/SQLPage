set url = sqlpage.set_variable('y', '2');
set path = sqlpage.path();
set x = json_extract(sqlpage.variables('get'), '$.x');

select 'text' as component,
    case
        when $x is not null AND ($url = $path || '?x=' || $x || '&y=2' OR $url = $path || '?y=2&x=' || $x) THEN 'It works !'
        when $x is null AND $url = $path || '?y=2' THEN 'It works !'
        else 'error: ' || $url
    end as contents;
