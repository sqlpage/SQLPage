select 'redirect' as component, '/404.sql' as link where sqlpage.path() <> '/404.sql';

select 'status_code' as component, 404 as status;
select 'http_header' as component, 'no-store, max-age=0' as "Cache-Control";
select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'hero' as component,
    'Not Found' as title,
    'Sorry, we couldn''t find the page you were looking for.' as description_md,
    '/your-first-sql-website/not_found.jpg' as image;