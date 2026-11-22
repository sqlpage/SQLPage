-- Preserve links from older guides to individual components.
select 'redirect' as component, sqlpage.link('component.sql', json_object('component', $component)) as link
where $component is not null;

select 'http_header' as component, 'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', 'SQLPage Documentation',
    'section', 'Documentation',
    'current', 'Documentation',
    'outline', '[{"title":"Overview","link":"#overview"},{"title":"Reference","link":"#reference"},{"title":"How a page works","link":"#how-it-works"},{"title":"Guides","link":"#guides"}]'
)) as properties;

select 'docs-intro' as component,
    'Documentation' as title,
    'Build your first website in SQL, or look up a component or function for the page you are writing.' as description,
    sqlpage.version() as version,
    '/assets/landing/assets/scene-first-frame.webp' as image,
    '/your-first-sql-website/' as link, 'Build your first website' as link_text;

select 'docs-links' as component, 'Reference' as title, 'reference' as id;
select 'Components' as title, 'layout-grid' as icon, '/components.sql' as link,
    format('Browse %d components with their parameters, SQL examples and previews.', (select count(*) from component)) as description,
    (select json_group_array(json_object('title', name, 'link', '/component.sql?component=' || name))
        from (select name from component where name in ('form', 'table', 'map', 'chart', 'button') order by name)) as links;
select 'Functions' as title, 'function' as icon, '/functions.sql' as link,
    format('Look up the arguments and behavior of %d built-in SQLPage functions.', (select count(*) from sqlpage_functions)) as description,
    (select json_group_array(json_object('title', name, 'link', '/functions.sql?function=' || name))
        from (select name from sqlpage_functions where name in ('fetch', 'link', 'run_sql', 'variables') order by name)) as links;

select 'title' as component, 'How a page works' as contents, 2 as level, 'how-it-works' as id;
select 'text' as component, 'Each SQL file is a page. Its first SELECT names a component and sets its top-level parameters. The rows returned by the next SELECT supply the row-level parameters. This list uses both.' as contents;
select 'docs-example' as component, 1 as number, 'A reading list' as title, true as preview,
    sqlpage.read_file_as_text('sqlpage/docs-intro-example.sql') as source;
select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-intro-example.sql') as properties;
select 'docs-example' as component, true as end_result;

select 'docs-links' as component, 'Guides' as title, 'guides' as id;
select title, link, description
from guide
order by sort_order;

select 'text' as component, 'Have a question? Ask on the [SQLPage forum](https://github.com/sqlpage/SQLPage/discussions).' as contents_md;
