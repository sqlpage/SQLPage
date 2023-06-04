select 'shell' as component,
    'SQLPage: get started!' as title,
    'database' as icon,
    '/' as link,
    'en-US' as lang,
    'Get started with SQLPage: short tutorial' as description,
    'documentation' as menu_item,
    'Poppins' as font;

SELECT 'hero' as component,
    'Let''s get started with SQLPage' as title,
    'Below is a short tutorial that will help you create your first web page in SQL.' as description;

SELECT 'list' as component,
    'Are you comfortable with command line applications ?' as title;

SELECT 'Yes, I can use the terminal' as title,
    'manual_setup.sql' as link,
    'I can type commands in a terminal and have used the command line before. I want the technical instructions.' as description,
    'black' as color,
    'prompt' as icon;
SELECT 'No, I want to do it the easy way' as title,
    'hosted.sql' as link,
    'I don''t want to have anything to do with scary hacker things. I will pay a monthly fee, and never have to configure a server myself.' as description,
    'green' as color,
    'mood-happy' as icon;