SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bubble' AS type, 4 AS marker;
WITH points(series, x, y, z) AS (VALUES ('A', 'Q1', 1, 30), ('A', 'Q2', 2, 30), ('B', 'Q2', 5, 70)) SELECT * FROM points;
