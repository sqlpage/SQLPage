SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'line' AS type, 4 AS marker;
WITH points(series, x, y, yline, label) AS (VALUES ('A', 'Q1', 1, NULL, NULL), ('A', 'Q2', 2, NULL, NULL), ('A', 'Q3', 3, NULL, NULL), (NULL, NULL, NULL, 2, 'limit')) SELECT * FROM points;
