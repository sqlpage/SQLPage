select 'http_header' as component,
    'public, max-age=300, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control",
    'User-Agent' as "Vary",
    '<https://sql-page.com/your-first-sql-website/>; rel="canonical"' as "Link";

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', 'Your first SQL website | SQLPage',
    'description', 'Create a SQLPage website from scratch, connect a database and build an interactive form.',
    'css', '/assets/docs/install.css',
    'section', 'Guides', 'current', 'Your first website',
    'outline', '[
        {"title":"Overview","link":"#overview"},
        {"title":"Install SQLPage","link":"#download"},
        {"title":"Build the website","link":"#tutorial"},
        {"title":"Connect a database","link":"#connect-to-a-custom-database"},
        {"title":"Handle form input","link":"#forms"},
        {"title":"Deploy your website","link":"#deploy"},
        {"title":"Related reference","link":"#reference"},
        {"title":"Online and video options","link":"#alternatives"}
    ]'
)) as properties;

-- Prefer a shared link's target; offer the OS picker when the browser cannot run a desktop release.
set user_agent = sqlpage.header('user-agent');
set selected_os = coalesce(
    (select os from install_option where os = lower($os) limit 1),
    case
        when lower($os) = 'any' or $user_agent like '%Mobile%' or $user_agent like '%Android%' then null
        when $user_agent like '%Windows%' then 'windows'
        when $user_agent like '%Macintosh%' then 'macos'
        when $user_agent like '%Linux%' then 'linux'
    end
);

select 'docs-intro' as component, true as compact, true as photograph,
    'Your first SQL website' as title,
    'Connect a database, display its rows, and let visitors add a record through a form.' as description,
    coalesce('get_started_' || $selected_os || '.webp', 'get_started.webp') as image;

set selected_method = select method from install_option
    where os = $selected_os
    order by method = coalesce(lower($method), '') desc, position
    limit 1;

with platforms(os, label, icon, position) as (values
    ('macos', 'macOS', 'brand-apple', 1),
    ('windows', 'Windows', 'brand-windows', 2),
    ('linux', 'Linux', 'terminal-2', 3)
), selection(os, method) as (values ($selected_os, $selected_method))
select 'docs-install' as component, command, download_url, download_label, notes,
    (select json_group_array(json_object(
        'label', p.label, 'icon', p.icon, 'active', p.os = $selected_os,
        'link', '?os=' || p.os || coalesce('&method=' || shared.method, '') || '#download'
    ) order by p.position)
    from platforms p left join install_option shared
        on shared.os = p.os and shared.method = $selected_method and shared.method = lower($method)
    ) as platforms,
    (select json_group_array(json_object(
        'label', label, 'active', method = $selected_method,
        'link', '?os=' || os || '&method=' || method || '#download'
    ) order by position)
    from install_option where os = $selected_os
    ) as methods
from selection left join install_option using (os, method);

-- This checked-in article has HTML heading IDs for direct links from the landing page.
select 'text' as component,
    sqlpage.read_file_as_text('your-first-sql-website/tutorial.md') as unsafe_contents_md,
    true as article,
    'tutorial' as id;

select 'docs-links' as component, 'Component and function reference' as title, 'reference' as id;
select 'Form and field options' as title, 'forms' as icon, '/component.sql?component=form' as link,
    'Add input types, validation and submit buttons to the form you built.' as description;
select 'Read request variables' as title, 'variable' as icon, '/functions.sql?function=variables' as link,
    'Read the URL and form values sent to a page with sqlpage.variables().' as description;

select 'title' as component, 'Online and video options' as contents, 2 as level, 'alternatives' as id;
select 'text' as component, '
- [Try SQLPage in your browser](https://editor.datapage.app) or [host your app with DataPage](https://datapage.app).
- [Watch the introduction](https://www.youtube.com/watch?v=9NJgH_-zXjY) or [follow the tutorial video series](https://www.youtube.com/watch?v=6D5D10v18b0&list=PLTue_qIAHxActQnLn_tHWZUNXziZTeraB).' as contents_md;
