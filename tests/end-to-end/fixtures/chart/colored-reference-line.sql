SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'line' AS type, 100 AS ymax, 4 AS marker;
WITH points(series, x, y, yline, label, color) AS (VALUES (NULL, NULL, NULL, 70, 'target', 'green'), ('A', 'Q1', 1, NULL, NULL, NULL), ('A', 'Q2', 2, NULL, NULL, NULL)) SELECT * FROM points;
