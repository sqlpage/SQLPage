SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bubble' AS type, 4 AS marker;
WITH points(series, x, y, color, z) AS (VALUES ('A', 'Q1', 1, 'red', 30), ('A', 'Q2', 2, 'green', 30)) SELECT * FROM points;
