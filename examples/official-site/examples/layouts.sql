select 'dynamic' as component, 
    json_patch(properties->0, json_object('layout', coalesce($layout, 'boxed'))) as properties
FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, '
# Layouts

SQLPage comes with a few built-in layouts that you can use to organize the general structure of your application.

Click on one of the layouts below to try it out.

For more information on how to use layouts, see the [shell component documentation](/documentation.sql?component=shell#component).
' as contents_md;

select 'list' as component, 'Available SQLPage shell layouts' as title;
select column1 as title, '?layout=' || lower(column1) as link, $layout = column1 as active, column3 as icon, column2 as description
from (VALUES
    ('Boxed', 'A compact layout with a fixed-width container. This is the default layout.', 'layout-distribute-vertical'),
    ('Horizontal', 'A full-width layout with a horizontal navigation bar.', 'layout-align-top'),
    ('Fluid', 'A full-width layout with a fluid container.', 'layout-distribute-horizontal')
) as t;