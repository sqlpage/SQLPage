select 'http_header' as component, '<https://sql-page.com/examples/>; rel="canonical"' as "Link";
select 'shell-docs' as component, 'Examples | SQLPage' as title, 'Examples' as section;

select 'title' as component, 'Examples' as contents;
select 'docs-links' as component, 'Try a live example' as title, 'live-examples' as id;
select column1 as title, column2 as icon, column3 as link, column4 as description
from (values
    ('Multi-step form', 'forms', '/examples/multistep-form/', 'Book a flight with a form that carries your choices from one page to the next.'),
    ('File uploads', 'upload', '/examples/handle_picture_upload.sql', 'Upload a picture and display it on the page.'),
    ('Password protection', 'lock', '/examples/authentication/', 'Sign in to a page that uses a database-backed session.'),
    ('Page layouts', 'layout', '/examples/layouts.sql', 'Switch between boxed, horizontal, and fluid layouts and try the sidebar.')
);

select 'title' as component, 'Example projects' as contents, 2 as level, 'projects' as id;
select 'text' as component, 'Choose a database to find a complete project and read its SQL on GitHub.' as contents;

select 'tab' as component, true as center;
select 'Show all examples' as title, 'All database examples' as description, '/examples/#projects' as link, $db is null as active;
select db_engine as title,
       format('%s database examples', db_engine) as description,
       sqlpage.link('/examples/', json_object('db', db_engine), 'projects') as link,
       $db=db_engine as active,
       case $db when db_engine then db_engine end as color
from example_cards
group by db_engine;

select 'card' as component;
select title, description,
    format('/examples/tabs/images/%s.svg', folder) as top_image,
    db_engine as color,
    'https://github.com/sqlpage/SQLPage/tree/main/examples/' || folder as link
from example_cards
where $db is null or $db = db_engine;

select 'text' as component, '[Browse all example projects on GitHub](https://github.com/sqlpage/SQLPage/tree/main/examples)' as contents_md;
