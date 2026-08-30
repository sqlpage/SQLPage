SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bar' AS type, TRUE AS horizontal, 4 AS marker;
WITH points(series, x, y, color) AS (VALUES ('Accounts', '30 days', 100, 'red'), ('Accounts', '60 days', 200, 'orange'), ('Accounts', '90 days', 300, 'green')) SELECT * FROM points;
