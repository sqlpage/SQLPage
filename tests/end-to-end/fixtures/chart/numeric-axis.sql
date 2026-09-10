SELECT
    'chart' AS component,
    'test-chart' AS id,
    'Every bar label equals its x value' AS title,
    'bar' AS type,
    TRUE AS labels;

WITH RECURSIVE x(x) AS (
    VALUES (1)
    UNION ALL
    SELECT x + 1 FROM x WHERE x < 12
)
SELECT 'A' AS series, x, x AS y FROM x
UNION ALL
SELECT 'B', x, x FROM x;
