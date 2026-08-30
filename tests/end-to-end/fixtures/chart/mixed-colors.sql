SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bar' AS type, 4 AS marker;
WITH points(series, x, y, color) AS (VALUES ('A', 'Q1', 1, NULL), ('A', 'Q2', 2, 'red')) SELECT * FROM points;
