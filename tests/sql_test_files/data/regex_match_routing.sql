set route = sqlpage.regex_match(
    '/categories/(?<category>[A-Za-z0-9_]+)/post/(?<id>[0-9]+)',
    '/categories/sql/post/42'
);

select
    '{"0":"/categories/sql/post/42","category":"sql","id":"42"}' as expected,
    $route as actual;
