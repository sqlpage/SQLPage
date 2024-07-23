delete from sqlpage_files
where path = $path and $confirm = 'yes'
returning 
    'redirect' as component,
    sqlpage.link(
        'index.sql',
        json_build_object('deleted', $path)
    ) as link;

select 'alert' as component,
    'Delete ' || $path || ' ?' as title,
    'Are you sure you want to delete ' || $path || '?' as description,
    'warning' as color,
    sqlpage.link('delete.sql', json_build_object('path', $path, 'confirm', 'yes')) as link,
    'Delete' as link_text;