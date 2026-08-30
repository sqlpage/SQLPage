SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'treemap' AS type, 4 AS marker;
WITH points(series, x, y) AS (VALUES ('North America', 'United States', 35), ('North America', 'Canada', 15), ('Europe', 'France', 30), ('Europe', 'Germany', 55)) SELECT * FROM points;
