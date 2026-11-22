select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', 'Components - SQLPage Documentation', 'section', 'Components'
)) as properties;

select 'docs-reference-heading' as component, 'overview' as id,
    'Documentation' as section, '/documentation.sql' as link, 'layout-grid' as icon,
    'Components' as title, 'Choose a component to see its parameters, SQL examples, and rendered output.' as description;

select 'docs-catalog' as component;
with entries as (
    select name, icon, ltrim(trim(description, char(10) || ' '), '# ') as description,
        case
            when name in ('big_number', 'card', 'carousel', 'chart', 'datagrid', 'list', 'map', 'table', 'timeline', 'tracking') then 'Display data'
            when name in ('alert', 'breadcrumb', 'button', 'facet', 'form', 'login', 'modal', 'pagination', 'status_bar', 'steps', 'tab', 'toast') then 'Interaction and navigation'
            when name in ('code', 'columns', 'divider', 'empty_state', 'foldable', 'hero', 'html', 'shell', 'text', 'title') then 'Layout and content'
            when name in ('cookie', 'csv', 'download', 'http_header', 'json', 'redirect', 'rss', 'status_code') then 'Requests and responses'
            else 'Server logic'
        end as category
    from component
)
select name, icon, category,
    row_number() over (partition by category order by name) = 1 as first_in_category,
    substr(description, 1, instr(description || char(10), char(10)) - 1) as summary_md,
    sqlpage.link('/component.sql', json_object('component', name)) as link
from entries order by category, name;
