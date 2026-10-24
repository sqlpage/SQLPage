SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bar' AS type, TRUE AS stacked, FALSE AS show_legend;
WITH points(series, x, y) AS (VALUES ('CPU', 'Mon', 10), ('CPU', 'Tue', 20), ('GPU', 'Mon', 5), ('GPU', 'Tue', 15)) SELECT * FROM points;
