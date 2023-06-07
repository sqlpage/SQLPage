-- This line, at the top of the page, tells web browsers to keep the page locally in cache once they have it.
select 'http_header' as component, 'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";
select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, 'SQLPage documentation' as title;
select '
Building an application with SQLPage is quite simple.
To create a new web page, just create a new SQL file. 
For each SELECT statement that you write, the data it returns will be analyzed and rendered to the user.
The two most important concepts in SQLPage are **components** and **parameters**.

 - **components** are small user interface elements that you can use to display your data in a certain way.
 - *top-level* **parameters** are the properties of these components, allowing you to customize their appearance and behavior.
 - *row-level* **parameters** constitute the data that you want to display in the components.

To select a component and set its top-level properties, you write the following SQL statement: 

```sql
SELECT ''component_name'' AS component, ''my value'' AS top_level_parameter_1;
```

Then, you can set its row-level parameters by writing a second SELECT statement:

```sql
SELECT my_column_1 AS row_level_parameter_1, my_column_2 AS row_level_parameter_2 FROM my_table;
```

This page documents all the components provided by default in SQLPage and their parameters.
Use this as a reference when building your SQL application.

If you have some frontend development experience, you can also create your own components, by placing
[`.handlebars`](https://handlebarsjs.com/guide/) files in a folder called `sqlpage/templates` at the root of your server.
[See example](https://github.com/lovasoa/SQLpage/blob/main/sqlpage/templates/list.handlebars).
' as contents_md;

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
            "description_md": ' || json_quote(description) || ',
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
        }, '||
        CASE component
            WHEN 'shell' THEN '{"component": "text", "contents": ""}'
            WHEN 'http_header' THEN '{ "component": "text", "contents": "" }'
            ELSE '
                {"component": "title", "level": 3, "contents": "Result"},
                {"component": "dynamic", "properties": ' || properties ||' }
            '
        END || '
    ]
    ' as properties
from example where component = $component;