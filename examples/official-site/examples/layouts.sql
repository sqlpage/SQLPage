select 
    'shell'                   as component,
    'SQLPage'                 as title,
    '/'                       as link,
    'layouts'                 as menu_item,
    'chart'                   as menu_item,
    'tabs'                    as menu_item,
    'hash_password'           as menu_item,
    COALESCE($layout,'boxed') as layout,
    'Documentation for the SQLPage low-code web application framework.' as description,
    'Poppins'                 as font,
    'layout'                    as icon,
    'https://cdn.jsdelivr.net/npm/prismjs@1/components/prism-core.min.js' as javascript,
    'https://cdn.jsdelivr.net/npm/prismjs@1/plugins/autoloader/prism-autoloader.min.js' as javascript,
    '/prism-tabler-theme.css' as css;

select 'text' as component, '
# Layouts

SQLPage comes with a few built-in layouts that you can use to organize the general structure of your application.

Click on one of the layouts below to try it out.
' as contents_md;

select 'list' as component, 'Available SQLPage shell layouts' as title;
select column1 as title, '?layout=' || lower(column1) as link, $layout = column1 as active, column3 as icon, column2 as description
from (VALUES
    ('Boxed', 'A compact layout with a fixed-width container. This is the default layout.', 'layout-distribute-vertical'),
    ('Horizontal', 'A full-width layout with a horizontal navigation bar.', 'layout-align-top'),
    ('Fluid', 'A full-width layout with a fluid container.', 'layout-distribute-horizontal')
) as t;