set route = sqlpage.regex_match(
    '/categories/(?<category>\w+)/post/(?<id>\d+)',
    '/categories/sql/post/42'
);

select
    '{"0":"/categories/sql/post/42","category":"sql","id":"42"}' as expected,
    $route as actual;
