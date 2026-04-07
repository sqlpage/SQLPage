SELECT
    'chart' AS component,
    'Legend Hidden' AS title,
    'line' AS type,
    FALSE AS show_legend;

SELECT 'Marketing' AS series, 2023 AS x, 30 AS y
UNION ALL
SELECT 'Marketing', 2024, 45
UNION ALL
SELECT 'Sales', 2023, 35
UNION ALL
SELECT 'Sales', 2024, 50;
