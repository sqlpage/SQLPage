create table
    example_cards as
select
    'Advanced Authentication' as title,
    'user-authentication' as folder, -- Has to exactly match the folder name in the /examples/ directory
    'postgres' as db_engine,
    'Build a secure user authentication system with login, signup, and in-database session management.' as description
union all
select
    'Authenticated CRUD',
    'CRUD - Authentication',
    'sqlite',
    'Complete Create-Read-Update-Delete operations with user authentication.'
union all
select
    'Image Gallery',
    'image gallery with user uploads',
    'sqlite',
    'Create an image gallery with user uploads and session management.'
union all
select
    'Developer UI',
    'SQLPage developer user interface',
    'postgres',
    'A web-based interface for managing SQLPage files and database tables.'
union all
select
    'Corporate Game',
    'corporate-conundrum',
    'sqlite',
    'An interactive multiplayer board game with real-time updates.'
union all
select
    'Roundest Pokemon',
    'roundest_pokemon_rating',
    'sqlite',
    'Demo app with a distinct non-default design, using custom HTML templates for everything.'
union all
select
    'Todo Application',
    'todo application (PostgreSQL)',
    'postgres',
    'A full-featured todo list application with PostgreSQL backend.'
union all
select
    'MySQL & JSON',
    'mysql json handling',
    'mysql',
    'Learn advanced JSON manipulation in MySQL to build advanced SQLPage applications.'
union all
select
    'Apache Web Server',
    'web servers - apache',
    'mysql',
    'Use an existing Apache httpd Web Server to expose your SQLPage application.'
union all
select
    'Sending Emails',
    'sending emails',
    'sqlite',
    'Use the fetch function to send emails (or interact with any other HTTP API).'
union all
select
    'Simple Website',
    'simple-website-example',
    'sqlite',
    'Basic website example with navigation and data management.'
union all
select
    'Geographic App',
    'PostGIS - using sqlpage with geographic data',
    'postgres',
    'Use SQLPage to create and manage geodata.'
union all
select
    'Multi-step form',
    'forms-with-multiple-steps',
    'sqlite',
    'Guide to the implementation of forms that spread over multiple pages.'
union all
select
    'Custom HTML & JS',
    'custom form component',
    'mysql',
    'Building a custom form component with a dynamic widget using HTML and javascript.'
union all
select
    'Splitwise Clone',
    'splitwise',
    'sqlite',
    'An expense tracker app to split expenses with your friends, with nice debt charts.'
union all
select
    'Advanced Forms with MS SQL Server',
    'microsoft sql server advanced forms',
    'sql server',
    'Forms with multi-value dropdowns, using SQL Server and its JSON functions.'
union all
select
    'Rich Text Editor',
    'rich-text-editor',
    'sqlite',
    'A rich text editor with bold, italic, lists, images, and more. It posts its contents as Markdown.'
;
