SELECT 'list' as component,
    'My points' as title;
SELECT label as title,
    'point.sql?id=' || id as link,
    'red' as color,
    'world-pin' as icon
FROM spatial_data;

SELECT 'form' AS component,
    'Add a point' AS title,
    'add_point.sql' AS action;

SELECT 'Label' AS name;
SELECT 'Latitude' AS name, 'number' AS type, 0.00000001 AS step;
SELECT 'Longitude' AS name, 'number' AS type, 0.00000001 AS step;