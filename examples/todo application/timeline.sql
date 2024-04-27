select 'dynamic' as component, sqlpage.run_sql('shell.sql') as properties;

select 
    'timeline' as component;
select 
    title,
    'todo_form.sql?todo_id=' || id as link,
    created_at as date,
    'calendar' as icon,
    'green' as color,
    printf('%d days ago', julianday('now') - julianday(created_at)) as description
from todos
order by created_at desc;