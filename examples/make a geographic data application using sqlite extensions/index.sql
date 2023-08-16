SELECT 'shell' as component,
    'Map' AS title,
    '/' as link,
    'book' as icon;

SELECT 'map' AS component,
    ST_Y(geom) AS latitude,
    ST_X(geom) AS longitude
FROM spatial_data
WHERE id = $id;

SELECT 'map' as component,
    'Points of interest' as title,
    2 as zoom,
    700 as height;
SELECT title,
    ST_Y(geom) AS latitude,
    ST_X(geom) AS longitude,
    'point.sql?id=' || id as link
FROM spatial_data;


SELECT 'list' as component,
    'My points' as title;
SELECT title,
    'point.sql?id=' || id as link,
    'red' as color,
    'world-pin' as icon
FROM spatial_data;

SELECT 'form' AS component,
    'Add a point' AS title,
    'add_point.sql' AS action;

SELECT 'Title' AS name;
SELECT 'Latitude' AS name, 'number' AS type, 0.00000001 AS step;
SELECT 'Longitude' AS name, 'number' AS type, 0.00000001 AS step;
SELECT 'Text' AS name, 'textarea' AS type;