-- This line, at the top of the page, tells web browsers to keep the page locally in cache once they have it.
select 'http_header' as component, 'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";
select 'shell' as component,
    'SQLPage documentation' as title,
    'file-database' as icon,
    '/' as link,
    'en-US' as lang,
    'SQLPage documentation: API reference listing all available components in the low-code web application framework' as description,
    'index' as menu_item;

select 'text' as component, 'SQLPage documentation' as title;
select 'Building an application with SQLPage is quite simple.' ||
    'To create a new web page, just create a new SQL file. ' ||
    'For each SELECT statement that you write, the data it returns will be analyzed and rendered to the user.';
select 'The two most important concepts in SQLPage are ' as contents;
select 'components' as contents, true as bold;
select ' and ' as contents;
select 'parameters' as contents, true as bold;
select '.' as contents;
select 'This page documents all the components that you can use in SQLPage and their parameters. ' ||
     'Use this as a reference when building your SQL application.' as contents;

select 'list' as component, 'components' as title;
select
    name as title,
    description,
    icon,
    '?component='||name||'#component' as link,
    $component = name as active
from component
order by name;

select 'text' as component,
    'The "'||$component||'" component' as title,
    'component' as id;
select description as contents from component where name = $component;

select 'title' as component, 3 as level, 'Top-level parameters' as contents where $component IS NOT NULL;
select 'card' as component, 3 AS columns where $component IS NOT NULL;
select
    name as title,
    (CASE WHEN optional THEN '' ELSE 'REQUIRED. ' END) || description as description,
    type as footer,
    CASE WHEN optional THEN 'lime' ELSE 'azure' END as color
from parameter where component = $component AND top_level
ORDER BY optional, name;


select 'title' as component, 3 as level, 'Row-level parameters' as contents
WHERE $component IS NOT NULL AND EXISTS (SELECT 1 from parameter where component = $component AND NOT top_level);
select 'card' as component, 3 AS columns where $component IS NOT NULL;
select
    name as title,
    (CASE WHEN optional THEN '' ELSE 'REQUIRED. ' END) || description as description,
    type as footer,
    CASE WHEN optional THEN 'lime' ELSE 'azure' END as color
from parameter where component = $component AND NOT top_level
ORDER BY optional, name;

select
    'dynamic' as component,
    '[
        {"component": "code"},
        {
            "title": "Example ' || (row_number() OVER ()) || '",
            "description": ' || json_quote(description) || ',
            "contents": ' || json_quote((
                select
                     group_concat(
                        'SELECT ' || char(10) ||
                            (
                                select group_concat(
                                    '    ' ||
                                    CASE typeof(value) 
                                        WHEN 'integer' THEN value::text
                                        ELSE quote(value::text)
                                    END ||
                                    ' as ' ||
                                    key
                                , ',' || char(10)
                                ) from json_each(top.value)
                            ) || ';',
                        char(10)
                     )
                from json_each(properties) AS top
        )) || '
        },
        {"component": "title", "level": 3, "contents": "Result"},
        {"component": "dynamic", "properties": ' || properties ||' }
    ]
    ' as properties
from example where component = $component;