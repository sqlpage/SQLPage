select 'http_header' as component,
    'public, max-age=300, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control",
    '<https://sql-page.com/your-first-sql-website/>; rel="canonical"' as "Link";

set os = COALESCE($os, case 
    when sqlpage.header('user-agent') like '%windows%' then 'windows'
    when sqlpage.header('user-agent') like '%x11; linux%' then 'linux'
    when sqlpage.header('user-agent') like '%x11; ubuntu; linux%' then 'linux'
    when sqlpage.header('user-agent') like '%x11; debian; linux%' then 'linux'
    when sqlpage.header('user-agent') like '%macintosh%' then 'macos'
    else 'any'
end);

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', 'Your first SQL website | SQLPage',
    'description', 'Create a SQLPage website from scratch, connect a database and build an interactive form.',
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

SET req = '{
    "url": "https://api.github.com/repos/sqlpage/SQLPage/releases/latest",
    "timeout_ms": 200
}';
SET api_results = sqlpage.fetch_with_meta($req);
SET sqlpage_version = COALESCE(json_extract($api_results, '$.json_body.tag_name'), '');

SELECT 'docs-intro' as component,
    true as photograph,
    'Your first SQL Website' as title,
    'Create a website from scratch. Connect a database, display its rows, and let visitors add a record through a form.' as description,
    case $os
        when 'linux' then 'get_started_linux.webp'
        when 'macos' then 'get_started_macos.webp'
        when 'windows' then 'get_started_windows.webp'
        else 'get_started.webp'
    end as image,
    CASE $os
        WHEN 'macos' THEN '#download'
        WHEN 'windows' THEN 'https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage-windows.zip'
        WHEN 'linux' THEN 'https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage-linux.tgz'
        ELSE 'https://github.com/sqlpage/SQLPage/releases'
    END AS link,
    CASE $os
        WHEN 'macos' THEN CONCAT('Install SQLPage ', $sqlpage_version, ' for macOS')
        WHEN 'windows' THEN CONCAT('Download SQLPage ', $sqlpage_version, ' for Windows')
        WHEN 'linux' THEN CONCAT('Download SQLPage ', $sqlpage_version, ' for Linux')
        ELSE CONCAT('Download SQLPage ', $sqlpage_version)
    END AS link_text;

select 'text' as component, 'You can also [try SQLPage in your browser](https://editor.datapage.app) or [watch the video tutorial](#alternatives).' as contents_md;

select 'text' as component,
    sqlpage.read_file_as_text(
        printf('your-first-sql-website/tutorial-install-%s.md',
            case
                when $os = 'windows' then 'windows'
                when $os = 'macos' then 'macos'
                else 'any'
            end
        )
    ) as contents_md,
    true as article,
    'download' as id;

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
