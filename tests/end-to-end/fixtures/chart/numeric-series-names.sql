SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bar' AS type, 4 AS marker;
WITH points(series, x, y) AS (VALUES (2024, 'Q1', 1), (2023, 'Q1', 2), ('total', 'Q1', 3)) SELECT * FROM points;
