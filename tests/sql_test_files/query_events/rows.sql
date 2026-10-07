select n, sqlpage.run_sql(path) as included
from (select 1 as n, 'tests/sql_test_files/query_events/child.sql' as path
      union all select 2 as n, 'tests/sql_test_files/query_events/child.sql' as path) source_rows
order by n;
set scalar_included = (select sqlpage.run_sql(path) from (select 'tests/sql_test_files/query_events/child.sql' as path) source_rows);
select $scalar_included as scalar_included;
select '[1]' as values_json, '[2]' as values_json,
    sqlpage.url_encode(name) as encoded, name as encoded
from (select 'a b' as name) source_rows;
