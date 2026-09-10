SELECT
    'chart' AS component,
    'test-chart' AS id,
    'Explicit numeric x intervals' AS title,
    'bar' AS type,
    2 AS xticks;

SELECT 'A' AS series, 1 AS x, 1 AS y
UNION ALL SELECT 'A', 4, 4
UNION ALL SELECT 'A', 12, 12;
