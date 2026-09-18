SELECT
    'chart' AS component,
    'test-chart' AS id,
    'Numeric horizontal bar categories' AS title,
    'bar' AS type,
    TRUE AS horizontal;

SELECT 1 AS x, 10 AS y
UNION ALL SELECT 4, 20
UNION ALL SELECT 12, 30;
