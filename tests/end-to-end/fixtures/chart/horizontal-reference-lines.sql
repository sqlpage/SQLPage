SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bar' AS type, TRUE AS horizontal, 100 AS ymax;
SELECT 90 AS yline, 'full' AS label;
SELECT 'db-1' AS xline, 'watched' AS label;
WITH points(x, y) AS (VALUES ('backup-1', 41), ('db-1', 88)) SELECT * FROM points;
