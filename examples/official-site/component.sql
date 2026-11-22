select 'redirect' as component,
    'component_not_found.sql' || coalesce('?component=' || sqlpage.url_encode($component), '') as link
where not exists (select 1 from component where name = $component);

select 'http_header' as component,
    'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control",
    printf('<%s>; rel="canonical"', sqlpage.link('component', json_object('component', $component))) as "Link";

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-reference.sql', json_object(
    'kind', 'component', 'name', $component
)) as properties;

select 'title' as component, 2 as level, 'Examples' as contents, 'examples' as id
where exists (select 1 from example where component = $component);

with examples as (
    -- HTTP headers and shells can only run at the start of a response.
    select *,
        case when component not in ('shell', 'http_header') then properties end as result,
        case when length(trim(description)) <= 120
            and glob('*[^a-zA-Z0-9 ,.''()-]*', trim(description)) = 0
            then trim(description) end as short_title
    from example_with_source where component = $component
)
select 'dynamic' as component, json_array(
    json_object(
        'component', 'docs-example', 'number', example_number,
        'title', short_title, 'description_md', case when short_title is null then description end,
        'source', source,
        'preview', result is not null
    ),
    json(coalesce(result, '[]')),
    -- Keep every native component of the example inside the same result panel.
    case when result is not null then json_object('component', 'docs-example', 'end_result', true)
        else json_array() end
) as properties
from examples order by example_id;
