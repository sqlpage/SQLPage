select 'http_header' as component, '</tabs/>; rel="canonical"' as "Link";
select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

drop table if exists example_cards;
create temporary table if not exists example_cards as 
select 'Advanced Authentication' as title, 'user-authentication' as folder, 'postgres' as db_engine, 'Build a secure user authentication system with login, signup, and in-database session management.' as description union all
select 'Authenticated CRUD', 'CRUD - Authentication', 'sqlite', 'Complete Create-Read-Update-Delete operations with user authentication.' union all
select 'Image Gallery', 'image gallery with user uploads', 'sqlite', 'Create an image gallery with user uploads and session management.' union all
select 'Developer UI', 'SQLPage developer user interface', 'postgres', 'A web-based interface for managing SQLPage files and database tables.' union all
select 'Corporate Game', 'corporate-conundrum', 'sqlite', 'An interactive multiplayer board game with real-time updates.' union all
select 'Todo Application', 'todo application (PostgreSQL)', 'sqlite', 'A full-featured todo list application with PostgreSQL backend.' union all
select 'MySQL & JSON', 'mysql json handling', 'mysql', 'Learn advanced JSON manipulation in MySQL to build advanced SQLPage applications.' union all
select 'Simple Website', 'simple-website-example', 'sqlite', 'Basic website example with navigation and data management.' union all
select 'Geographic App', 'PostGIS - using sqlpage with geographic data', 'postgres', 'Use SQLPage to create and manage geodata.' union all
select 'Multi-step form', 'forms-with-multiple-steps', 'sqlite', 'Guide to the implementation of forms that spread over multiple pages.' union all
select 'Custom HTML & JS', 'custom form component', 'mysql', 'Building a custom form component with a dynamic widget using HTML and javascript.' union all
select 'Advanced Forms with MS SQL Server', 'microsoft sql server advanced forms', 'sql server', 'Forms with multi-value dropdowns, using SQL Server and its JSON functions.';

select 'tab' as component, true as center;
select 'Show all examples' as title, 'All database examples' as description, '?' as link, $db is null as active;
select db_engine as title,
       format('%s database examples', db_engine) as description,
       format('?db=%s', db_engine) as link,
       $db=db_engine as active,
       case $db when db_engine then db_engine end as color
from example_cards
group by db_engine;

select 'card' as component;
select title, description,
    format('images/%s.svg', folder) as top_image,
    db_engine as color,
    'https://github.com/sqlpage/SQLPage/tree/main/examples/' || folder as link
from example_cards
where $db is null or $db = db_engine;

select 'text' as component, 'See [source code on GitHub](https://github.com/lovasoa/SQLpage/blob/main/examples/official-site/examples/tabs/)' as contents_md;