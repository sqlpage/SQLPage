SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bar' AS type, 4 AS marker;
WITH points(series, x, y) AS (VALUES ('A', 'X2', 10), ('A', 'X3', 30), ('B', 'X1', 25), ('B', 'X2', 20)) SELECT * FROM points;
