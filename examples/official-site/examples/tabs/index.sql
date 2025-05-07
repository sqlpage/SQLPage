select 'http_header' as component, '<https://sql-page.com/examples/tabs/>; rel="canonical"' as "Link";

select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', 'SQLPage - SQL website examples',
    'description', 'These small focused examples each illustrate one feature of the SQLPage website builder.'
)) as properties
FROM example WHERE component = 'shell' LIMIT 1;

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

select 'text' as component, 'See [source code on GitHub](https://github.com/sqlpage/SQLPage/blob/main/examples/official-site/examples/tabs/)' as contents_md;
