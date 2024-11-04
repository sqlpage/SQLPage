select 'http_header' as component, 'noindex' as "X-Robots-Tag";

select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 
    'hero'                 as component,
    'Not found'              as title,
    'Sorry, the component you were looking for does not exist.' as description_md,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/2/2b/Sad_clown.jpg/640px-Sad_clown.jpg' as image,
    '/documentation.sql'   as link,
    'Back to the documentation' as link_text;

-- Friendly message after an XSS or SQL injection attempt
set attack = CASE WHEN
    $component LIKE '%<%' or $component LIKE '%>%' or $component LIKE '%/%' or $component LIKE '%;%'
    or $component LIKE '%--%' or $component LIKE '%''%' or $component LIKE '%(%'
THEN 'attacked' END;

select 
    'alert'                    as component,
    'A note about security'    as title,
    'alert-triangle'           as icon,
    'teal'                     as color,
    TRUE              as important,
    'SQLPage takes secutity very seriously.
Fiddling with the URL to try to access data you are not supposed to see, or to
trigger a SQL or javacript injection, should never work.

However, if you think you have found a security issue, please
report it and we will fix it as soon as possible.
' as description
where $attack = 'attacked';
select 'safety.sql' as link, 'More about SQLPage security' as title where $attack='attacked';
select 'https://github.com/lovasoa/SQLpage/security' as link, 'Report a vulnerability' as title where $attack='attacked';