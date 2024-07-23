select 'title' as component,  $table_name as contents;
select 'table' as component, true as search, true as sort;

select 'dynamic' as component, t as properties
from table_contents($table_name) t
LIMIT 1000;

select 'alert' as component,
    CASE
        WHEN COUNT(*) = 0 THEN 'The table is empty.'
        WHEN COUNT(*) > 1000 THEN 'Only the first 1000 rows are shown.'
    END as description,
    'info' as color
from table_contents($table_name)
HAVING NOT COUNT(*)  BETWEEN 1 AND 1000;