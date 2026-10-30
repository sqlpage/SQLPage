SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'line' AS type, TRUE AS time, 100 AS ymax;
SELECT 70 AS yline, 'target' AS label;
SELECT 90 AS yline, 100 AS yline_end, 'throttling' AS label;
WITH points(x, y) AS (VALUES ('2024-05-01T08:00:00Z', 52), ('2024-05-01T11:00:00Z', 83), ('2024-05-01T14:00:00Z', 63)) SELECT * FROM points;
