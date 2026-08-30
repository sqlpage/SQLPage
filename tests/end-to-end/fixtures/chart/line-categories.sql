SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'line' AS type, 4 AS marker;
WITH points(series, x, y) AS (VALUES ('A', 'Q1', 1), ('A', 'Q2', 2), ('A', 'Q3', 3), ('B', 'Q2', 20), ('B', 'Q3', 30)) SELECT * FROM points;
