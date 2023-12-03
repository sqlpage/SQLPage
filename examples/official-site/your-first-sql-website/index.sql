select 'http_header' as component,
    'public, max-age=300, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

select 'shell' as component,
    'Your SQL Website' as title,
    'database' as icon,
    '/' as link,
    'en-US' as language,
    'Get started with SQLPage: short tutorial for making a SQL-only website' as description,
    'documentation' as menu_item,
    20 as font_size,
    'Poppins' as font;

SELECT 'hero' as component,
    'Your first SQL Website' as title,
    'Let''s create your first website in SQL together, from downloading SQLPage to publishing your site online.' as description,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/c/c4/Backlit_keyboard.jpg/1024px-Backlit_keyboard.jpg' as image,
    'https://replit.com/@pimaj62145/SQLPage#index.sql' as link,
    'Follow this tutorial online' as link_text;

SELECT 'alert' as component,
    'Afraid of the setup ? Do it the easy way !' as title,
    'mood-happy' as icon,
    'teal' as color,
    'You don’t want to have anything to do with scary hacker things ? You can use a preconfigured SQLPage  hosted on our servers, and **never have to configure a server** yourself.' as description_md,
    'hosted.sql' AS link,
    'Try SQLPage cloud' as link_text;

select 'text' as component, sqlpage.read_file_as_text('your-first-sql-website/tutorial.md') as contents_md;