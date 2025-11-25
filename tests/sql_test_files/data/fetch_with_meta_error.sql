set res = sqlpage.fetch_with_meta('http://not-a-real-url');

select '"error":"Request failed' as expected_contains, $res as actual;
