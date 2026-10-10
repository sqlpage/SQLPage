SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'pie' AS type, TRUE AS labels;
WITH points(label, value) AS (VALUES ('Yes', 65), ('No', 35)) SELECT * FROM points;
