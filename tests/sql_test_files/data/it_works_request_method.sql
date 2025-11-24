set actual = sqlpage.request_method();
select 'GET' as expected,
    coalesce($actual, 'NULL') as actual;
