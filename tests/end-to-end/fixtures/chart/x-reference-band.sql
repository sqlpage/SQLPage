SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'area' AS type, TRUE AS time;
SELECT '2024-05-01T10:00:00Z' AS xline, 'deploy' AS label;
SELECT '2024-05-01T11:30:00Z' AS xline, '2024-05-01T13:00:00Z' AS xline_end, 'incident' AS label;
WITH points(x, y) AS (VALUES ('2024-05-01T08:00:00Z', 120), ('2024-05-01T12:00:00Z', 512), ('2024-05-01T15:00:00Z', 133)) SELECT * FROM points;
