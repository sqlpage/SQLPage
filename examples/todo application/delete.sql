delete from todos
where id = $todo_id and $confirm = 'yes'
returning
    'redirect' as component,
    '/' as link;

select 'dynamic' as component, sqlpage.run_sql('shell.sql') as properties;

select
    'alert' as component,
    'red' as color,
    'Confirm deletion' as title,
    'Are you sure you want to delete the following todo item ?

> ' || title as description_md,
    '?todo_id=' || $todo_id || '&confirm=yes' as link,
    'Delete' as link_text
from todos where id = $todo_id;
