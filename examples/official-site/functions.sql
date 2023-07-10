select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, 'SQLPage built-in functions' as title;
select '
In addition to normal SQL functions supported by your database,
SQLPage provides a few special functions to help you extract data from user requests.

These functions are special, because they are not executed inside your database,
but by SQLPage itself before sending the query to your database.
Thus, they require all the parameters to be known at the time the query is sent to your database.
Function parameters cannot reference columns from the rest of your query.
' as contents_md;

select 'list' as component, 'SQLPage functions' as title;
select name as title,
    icon,
    '?function=' || name || '#function' as link,
    $function = name as active
from sqlpage_functions
order by name;

select 'text' as component,
        'The sqlpage.' || $function || ' function' as title,
        'function' as id
    where $function IS NOT NULL;

select 'text' as component;
select 'Introduced in SQLPage ' || introduced_in_version || '.' as contents, 1 as size from sqlpage_functions where name = $function;

SELECT description_md as contents_md FROM sqlpage_functions WHERE name = $function;

select 'title' as component, 3 as level, 'Parameters' as contents where $function IS NOT NULL AND EXISTS (SELECT 1 from sqlpage_function_parameters where "function" = $function);
select 'card' as component, 3 AS columns where $function IS NOT NULL;
select
    name as title,
    description_md as description,
    type as footer,
    'azure' as color
from sqlpage_function_parameters where "function" = $function
ORDER BY "index";