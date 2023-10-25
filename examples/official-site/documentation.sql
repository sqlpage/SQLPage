-- This line, at the top of the page, tells web browsers to keep the page locally in cache once they have it.
select 'http_header' as component, 'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";
select 
    'dynamic' as component,
    json_set(
        properties,
        '$[0].title',
        'SQLPage components' || COALESCE(': ' || $component, ' documentation')
    ) as properties
FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, format('SQLPage v%s documentation', sqlpage.version()) as title;
select '
If you are completely new to SQLPage, you should start by reading the [get started tutorial](get%20started.sql),
which will guide you through the process of creating your first SQLPage application.

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
If at any point you need help, you can ask for it on the [SQLPage forum](https://github.com/lovasoa/SQLpage/discussions).

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

select 'text' as component;
select format('Introduced in SQLPage v%s.', introduced_in_version) as contents, 1 as size
from component
where name = $component and introduced_in_version IS NOT NULL;

select 'title' as component, 3 as level, 'Top-level parameters' as contents where $component IS NOT NULL;
select 'card' as component, 3 AS columns where $component IS NOT NULL;
select
    name as title,
    (CASE WHEN optional THEN '' ELSE 'REQUIRED. ' END) || description as description,
    type as footer,
    CASE type 
        WHEN 'COLOR' THEN 'https://tabler.io/docs/base/colors'
        WHEN 'ICON' THEN 'https://tabler-icons.io/'
    END AS footer_link,
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
    CASE type 
        WHEN 'COLOR' THEN 'https://tabler.io/docs/base/colors'
        WHEN 'ICON' THEN 'https://tabler-icons.io/'
    END AS footer_link,
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
            "language": "sql",
            "contents": ' || json_quote((
                select
                     group_concat(
                        'select ' || char(10) ||
                            (
                                with t as (select * from json_tree(top.value)),
                                key_val as (select
                                        CASE t.type 
                                            WHEN 'integer' THEN t.atom
                                            WHEN 'real' THEN t.atom
                                            WHEN 'true' THEN 'TRUE'
                                            WHEN 'false' THEN 'FALSE'
                                            WHEN 'null' THEN 'NULL'
                                            ELSE quote(t.value)
                                        END as key,
                                        CASE parent.fullkey
                                            WHEN '$' THEN t.key
                                            ELSE parent.key
                                        END as val
                                    from t inner join t parent on parent.id = t.parent
                                    where t.atom is not null
                                ),
                                key_val_padding as (select
                                    key,
                                    val,
                                    1 + max(0, max(case when length(key) < 30 then length(key) else 0 end) over () - length(key)) as padding
                                    from key_val
                                )
                                select group_concat(
                                    format('    %s%.*cas %s', key, padding, ' ', val),
                                     ',' || char(10)
                                ) from key_val_padding
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
from example where component = $component AND properties IS NOT NULL;

SELECT 'title' AS component, 3 AS level, 'Examples' AS contents
WHERE EXISTS (SELECT 1 FROM example WHERE component = $component AND properties IS NULL);
SELECT 'text' AS component, description AS contents_md
FROM example WHERE component = $component AND properties IS NULL;