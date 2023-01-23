-- This line, at the top of the page, tells web browsers to keep the page locally in cache once they have it.
select 'http_header' as component, 'max-age=3600' as "Cache-Control";
select
    'SQLPage documentation' as title,
    '/' as link,
    'en-US' as lang,
    'Documentation for the SQLPage low-code web application framework.' as description;


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
from component;

select 'text' as component,
    'The "'||$component||'" component' as title,
    'component' as id;
select description as contents from component where name = $component;

select 'title' as component, 3 as level, 'Parameters' as contents where $component IS NOT NULL;
select 'card' as component, 3 AS columns where $component IS NOT NULL;
select
    name || (CASE WHEN top_level THEN ' (top-level)' ELSE '' END) as title,
    (CASE WHEN optional THEN '' ELSE 'REQUIRED. ' END) || description as description,
    type as footer,
    CASE WHEN top_level THEN 'lime' ELSE 'azure' END || CASE WHEN optional THEN '-lt' ELSE '' END as color
from parameter where component = $component
ORDER BY (NOT top_level), optional, name;

select
    'dynamic' as component,
    json_array(
        json_object('component', 'code'),
        json_object(
            'title', 'Example ' || (row_number() OVER ()),
            'description', description,
            'contents', (
                select
                     group_concat(
                        'SELECT ' || x'0A' ||
                            (
                                select group_concat(
                                    '    ' || quote(value) || ' as ' || key, ',' || x'0A'
                                ) from json_each(top.value)
                            ) || ';',
                        x'0A'
                     )
                from json_each(properties) AS top
            )
        ),
        json_object('component', 'title', 'level', 3, 'contents', 'Result'),
        json_object('component', 'dynamic', 'properties', properties)
    ) as properties
from example where component = $component;