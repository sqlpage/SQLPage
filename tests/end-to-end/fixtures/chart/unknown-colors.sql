SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bar' AS type, 4 AS marker;
WITH points(series, x, y, color) AS (VALUES ('A', 'Q1', 1, '#ff0000'), ('A', 'Q2', 2, 'chartreuse')) SELECT * FROM points;
