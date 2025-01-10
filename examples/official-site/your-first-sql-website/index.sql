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

-- Fetch the page title and header from the database
select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', 'SQL to Website - Tutorial',
    'description', 'Convert your SQL database into a website in minutes. In this 5-minute guide, we will create a simple website from scratch, and learn the basics of SQLPage.'
)) as properties
FROM example WHERE component = 'shell' LIMIT 1;

SELECT 'hero' as component,
    'Your first SQL Website' as title,
    '[SQLPage](/) is a free tool for building data-driven apps quickly.

Let''s create a simple website with a database from scratch, to learn SQLPage basics.' as description_md,
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
        WHEN 'macos' THEN 'Install SQLPage using Homebrew'
        WHEN 'windows' THEN 'Download SQLPage for Windows'
        WHEN 'linux' THEN 'Download SQLPage for Linux'
        ELSE 'Download SQLPage'
    END AS link_text;

SELECT 'alert' as component,
    'Afraid of the setup ? Do it the easy way !' as title,
    'mood-happy' as icon,
    'teal' as color,
    'You don’t want to install anything on your computer ?
    You can use a preconfigured SQLPage  hosted on our servers, and get your app online in minutes, without **ever having to configure a server** yourself.' as description_md,
    'https://datapage.app' AS link,
    'Host your app on our servers' as link_text;
select 'https://editor.datapage.app' as link, 'Try SQLPage from your browser' as title, 'teal' as color;

SELECT 'alert' as component,
    'Do you prefer videos ?' as title,
    'brand-youtube' as icon,
    'purple' as color,
    'We made videos to introduce you to SQLPage. You can watch them on YouTube. The videos cover everything from the underlying technology to the philosophy behind SQLPage to the actual steps to create your first website.' as description_md,
    'https://www.youtube.com/watch?v=9NJgH_-zXjY' AS link,
    'Watch the introduction video' as link_text;
select 'https://www.youtube.com/watch?v=6D5D10v18b0&list=PLTue_qIAHxActQnLn_tHWZUNXziZTeraB' as link, 'Tutorial video series' as title;

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

select 'text' as component,
        sqlpage.read_file_as_text('your-first-sql-website/tutorial.md') as contents_md,
        true as article,
        'tutorial' as id;
