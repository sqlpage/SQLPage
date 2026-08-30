SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'line' AS type, 4 AS marker;
WITH points(series, x, y, xline, yline) AS (VALUES ('A', 'Q1', 1, NULL, NULL), ('A', 'Q2', 2, NULL, NULL), ('A', 'Q3', 3, NULL, NULL), (NULL, NULL, NULL, NULL, 2), (NULL, NULL, NULL, 'Q2', NULL)) SELECT * FROM points;
