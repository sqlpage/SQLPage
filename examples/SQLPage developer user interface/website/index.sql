select
    'list' as component,
    'Website files' as title;

select
    path as title,
    path as link,
    sqlpage.link ('edit.sql', json_build_object ('path', path)) as edit_link,
    sqlpage.link ('delete.sql', json_build_object ('path', path)) as delete_link
from
    sqlpage_files;

select
    'Create new file' as title,
    'edit.sql' as link,
    'file-plus' as icon,
    'green' as color;

select 'list' as component,
    'Database tables' as title;

select
    table_name as title,
    sqlpage.link ('view_table.sql', json_build_object('table_name', table_name)) as link
from
    information_schema.tables
where
    table_schema = 'public'
    and table_type = 'BASE TABLE';