SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'line' AS type, 4 AS marker;
WITH points(series, x, y) AS (VALUES ('Coding', 'Mon', 6), ('Coding', 'Tue', 4), ('Coding', 'Wed', 7)) SELECT * FROM points;
