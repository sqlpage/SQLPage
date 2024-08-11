select 'http_header' as component,
    'public, max-age=300, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

set is_windows = COALESCE($is_windows, sqlpage.header('user-agent') like '%windows%');
set is_linux = COALESCE($is_linux, sqlpage.header('user-agent') like '%x11; linux%');
set is_macos = COALESCE($is_macos, sqlpage.header('user-agent') like '%macintosh%');

SELECT 'hero' as component,
    'Your first SQL Website' as title,
    'Let''s create your first website in SQL together, from downloading SQLPage to connecting it to your database, to making a web app' as description,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/c/c4/Backlit_keyboard.jpg/1024px-Backlit_keyboard.jpg' as image,
    'https://github.com/lovasoa/SQLpage/releases'|| case
        when $is_windows then '/latest/download/sqlpage-windows.zip'
        when $is_linux then '/latest/download/sqlpage-linux.tgz'
        else ''
    end as link,
    'Download SQLPage' || case
        when $is_windows then ' for Windows'
        when $is_linux then ' for Linux'
        else ''
    end as link_text;

SELECT 'alert' as component,
    'Afraid of the setup ? Do it the easy way !' as title,
    'mood-happy' as icon,
    'teal' as color,
    'You don’t want to have anything to do with scary hacker things ?
    You can use a preconfigured SQLPage  hosted on our servers, and **never have to configure a server** yourself.' as description_md,
    'https://replit.com/@pimaj62145/SQLPage#index.sql' AS link,
    'Try SQLPage from your browser' as link_text;
select 'https://datapage.app' as link, 'Host your app on our servers' as title, 'teal' as color;
SELECT 'alert' as component,
    'Do you prefer videos ?' as title,
    'brand-youtube' as icon,
    'purple' as color,
    'I made a video to introduce you to SQLPage. You can watch it on YouTube. The video covers everything from the underlying technology to the philosophy behind SQLPage to the actual steps to create your first website.' as description_md,
    'https://www.youtube.com/watch?v=9NJgH_-zXjY' AS link,
    'Watch the introduction video' as link_text;

select 'text' as component, sqlpage.read_file_as_text(printf('your-first-sql-website/tutorial-install-%s.md',
    case
        when $is_windows then 'windows'
        when $is_macos then 'macos'
        else 'any'
    end
)) as contents_md, 'download' as id;
select 'text' as component, sqlpage.read_file_as_text('your-first-sql-website/tutorial.md') as contents_md;