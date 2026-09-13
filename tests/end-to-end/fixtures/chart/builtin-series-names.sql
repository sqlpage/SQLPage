SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bar' AS type, 4 AS marker;
WITH points(series, x, y) AS (VALUES ('toString', 'Q1', 1), ('constructor', 'Q1', 2)) SELECT * FROM points;
