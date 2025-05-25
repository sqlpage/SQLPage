select 'http_header' as component,
    printf('<%s>; rel="canonical"',
            iif($function is not null, sqlpage.link('functions', json_object('function', $function)), 'functions.sql')
    ) as "Link";

select 'dynamic' as component, properties
FROM example WHERE component = 'shell' LIMIT 1;

select 'breadcrumb' as component;
select 'SQLPage' as title, '/' as link, 'Home page' as description;
select 'Functions' as title, '/functions.sql' as link, 'List of all functions' as description;
select $function as title, sqlpage.link('functions.sql', json_object('function', $function)) as link where $function IS NOT NULL;

select 'text' as component, 'SQLPage built-in functions' as title where $function IS NULL;
select '
In addition to normal SQL functions supported by your database,
SQLPage provides a few special functions to help you extract data from user requests.

These functions are special, because they are not executed inside your database,
but by SQLPage itself before sending the query to your database.
Thus, they require all the parameters to be known at the time the query is sent to your database.
Function parameters cannot reference columns from the rest of your query.
The only case when you can call a SQLPage function with a parameter that is not a constant is when it appears at the top level of a `SELECT` statement.
For example, `SELECT sqlpage.url_encode(url) FROM t` is allowed because SQLPage can execute `SELECT url FROM t` and then apply the `url_encode` function to each value.
' as contents_md where $function IS NULL;

select 'list' as component, 'SQLPage functions' as title where $function IS NULL;
select name as title,
    icon,
    '?function=' || name || '#function' as link,
    $function = name as active
from sqlpage_functions
where $function IS NULL
order by name;

select 'text' as component, 'sqlpage.' || $function || '(' || string_agg(name, ', ') || ')' as title, 'function' as id
from sqlpage_function_parameters where $function IS NOT NULL and "function" = $function;

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

select 
    'button' as component,
    'sm'     as size,
    'pill'   as shape;
select
    name as title,
    icon,
    sqlpage.link('functions.sql', json_object('function', name)) as link
from sqlpage_functions
where $function IS NOT NULL
order by name;