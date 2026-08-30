SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'pie' AS type, 4 AS marker;
WITH points(series, x, y, color) AS (VALUES ('A', 'Q1', 1, 'red'), ('A', 'Q2', 2, 'green')) SELECT * FROM points;
