-- ensure that the component exists and do not render this page if it does not
select 'redirect' as component,
    'component_not_found.sql?component=' || sqlpage.url_encode($component) as link
where $component is not null and not exists (select 1 from component where name = $component);

-- This line, at the top of the page, tells web browsers to keep the page locally in cache once they have it.
select 'http_header' as component, 'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', coalesce($component || ' - ', '') || 'SQLPage Documentation'
)) as properties
FROM example WHERE component = 'shell' LIMIT 1;

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
    (CASE WHEN optional THEN '' ELSE 'REQUIRED. ' END) || description_md as description_md,
    type as footer,
    CASE type 
        WHEN 'COLOR' THEN '/colors.sql'
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
    (CASE WHEN optional THEN '' ELSE 'REQUIRED. ' END) || description_md as description_md,
    type as footer,
    CASE type 
        WHEN 'COLOR' THEN '/colors.sql'
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
                                            WHEN 'object' THEN 'JSON(' || quote(t.value) || ')'
                                            WHEN 'array' THEN 'JSON(' || quote(t.value) || ')'
                                            ELSE quote(t.value)
                                        END as val,
                                        CASE parent.fullkey
                                            WHEN '$' THEN t.key
                                            ELSE parent.key
                                        END as key
                                    from t inner join t parent on parent.id = t.parent
                                    where ((parent.fullkey = '$' and t.type != 'array') 
                                        or (parent.type = 'array' and parent.path = '$'))
                                ),
                                key_val_padding as (select
                                    CASE 
                                        WHEN (key LIKE '% %') or (key LIKE '%-%') THEN
                                                format('"%w"', key)
                                        ELSE
                                            key
                                    END as key,
                                    val,
                                    1 + max(0, max(case when length(val) < 30 then length(val) else 0 end) over () - length(val)) as padding
                                    from key_val
                                )
                                select group_concat(
                                    format('    %s%.*cas %s', val, padding, ' ', key),
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


select 'title' as component, 2 as level, 'See also: other components' as contents;
select 
    'button' as component,
    'sm'     as size,
    'pill'   as shape;
select
    name as title,
    icon,
    sqlpage.link('component.sql', json_object('component', name)) as link
from component
order by name;