select 'redirect' as component,
    'component_not_found.sql' || coalesce('?component=' || sqlpage.url_encode($component), '') as link
where not exists (select 1 from component where name = $component);

select 'http_header' as component,
    'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control",
    printf('<%s>; rel="canonical"', sqlpage.link('component', json_object('component', $component))) as "Link";

set outline = (
    select json_group_array(json_object('title', title, 'link', link))
    from (
        select 'Overview' as title, '#component' as link
        union all select 'Top-level parameters', '#top-level-parameters' where exists
            (select 1 from parameter where component = $component and top_level)
        union all select 'Row-level parameters', '#row-level-parameters' where exists
            (select 1 from parameter where component = $component and not top_level)
        union all select 'Examples', '#examples' where exists
            (select 1 from example where component = $component)
    )
);

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', $component || ' - SQLPage Documentation', 'section', 'Components',
    'current', $component, 'outline', $outline
)) as properties;

select 'docs-reference-heading' as component, 'component' as id,
    'Component reference' as section, '/components.sql' as link,
    'The ' || name || ' component' as title, icon, introduced_in_version as version,
    case when exists (select 1 from example where component = $component) then '#examples' end as jump_link,
    'View examples' as jump_text
from component where name = $component;

select 'text' as component, true as article, description as contents_md
from component where name = $component;

select 'docs-parameters' as component, 'Top-level parameters' as title, 'top-level-parameters' as id,
    'Set these in the SELECT that names the component.' as description
where exists (select 1 from parameter where component = $component and top_level);
select name, type, not optional as required, coalesce(description_md, description) as description_md
from parameter where component = $component and top_level order by optional, name;

select 'docs-parameters' as component, 'Row-level parameters' as title, 'row-level-parameters' as id,
    'Return these columns in the rows that follow the component.' as description
where exists (select 1 from parameter where component = $component and not top_level);
select name, type, not optional as required, coalesce(description_md, description) as description_md
from parameter where component = $component and not top_level order by optional, name;

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
        'component', 'docs-example', 'number', row_number() over (order by example_id),
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
