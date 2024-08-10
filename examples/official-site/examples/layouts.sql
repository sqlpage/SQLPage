set layout = coalesce($layout, 'boxed');
set sidebar = coalesce($sidebar, 0);

select 'dynamic' as component, 
    json_patch(properties->0, 
        json_object(
            'layout', $layout,
            'sidebar', $sidebar = 1
        )
    ) as properties
FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, '
# Layouts

SQLPage comes with a few built-in layouts that you can use to organize the general structure of your application.
The menu items are displayed horizontally at the top of the page by default, which you can change to a vertical sidebar
with `true as sidebar`.
The default `layout` is `''boxed''`, which has a fixed-width container for the main content area.
You can also choose between a `''horizontal''` layout with a larger container and a `''fluid''` layout that spans the full width of the page.
When using a sidebar, the page layout will automatically switch to a fluid layout.

Click on one of the layouts below to try it out.

For more information on how to use layouts, see the [shell component documentation](/documentation.sql?component=shell#component).
' as contents_md;

select 'list' as component, 'Available SQLPage shell layouts' as title;
select 
    column1 as title,
    sqlpage.link('', json_object('layout', lower(column1), 'sidebar', $sidebar)) as link,
    $layout = lower(column1) as active,
    column3 as icon,
    column2 as description
from (VALUES
    ('Boxed', 'A compact layout with a fixed-width container. This is the default layout.', 'layout-distribute-vertical'),
    ('Horizontal', 'A full-width layout with a horizontal navigation bar.', 'layout-align-top'),
    ('Fluid', 'A full-width layout with a fluid container.', 'layout-distribute-horizontal')
) as t;

select 'list' as component, 'Available Menu layouts' as title;
select
    column1 as title,
    sqlpage.link('', json_object('layout', $layout, 'sidebar', column1 = 'Sidebar')) as link,
    (column1 = 'Sidebar' AND $sidebar = 1) OR (column1 = 'Horizontal' AND $sidebar = 0) as active,
    column2 as description,
    column3 as icon
from (VALUES
    ('Horizontal', 'Display menu items horizontally at the top of the page.', 'layout-navbar'),
    ('Sidebar', 'Display menu items vertically on the left side of the page.', 'layout-sidebar')
) as t;

select 'code' as component;
select 'SQL code to use' as title, 'sql' as language, printf('select
  ''shell'' as component,
  ''%s'' as layout,
  %s as sidebar;
', $layout, case when $sidebar then 'true' else 'false' end) as contents;