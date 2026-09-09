SELECT
    'chart' AS component,
    'test-chart' AS id,
    'Irregular numeric x values' AS title,
    'bar' AS type,
    TRUE AS labels;

SELECT 'A' AS series, 0.25 AS x, 1 AS y
UNION ALL SELECT 'A', 0.5, 2
UNION ALL SELECT 'A', 3, 3;
