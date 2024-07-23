select 'shell' as component,
    'js/code-editor.js' as javascript;

select
    'form' as component,
    COALESCE('Editing ' || $path, 'New page') as title,
    'insert_file.sql' as action;

select
    'path' as name,
    'text' as type,
    'Name' as label,
    $path as value,
    'test.sql' as placeholder;

select
    'textarea' as type,
    'contents' as name,
    'code-editor' as id,
    'Contents' as label,
    (select contents from sqlpage_files where path = $path) as value,
    'SELECT ''text'' as component,
    ''Hello, world!'' as contents;' as placeholder;