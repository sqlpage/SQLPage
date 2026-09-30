SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title, 'bubble' AS type,
  'Weekday' AS xtitle, 'Hours' AS ytitle, 'Weight' AS ztitle;
WITH points(series, x, y, z) AS (VALUES ('Coding', 'Mon', 6, 30), ('Coding', 'Tue', 4, 30)) SELECT * FROM points;
