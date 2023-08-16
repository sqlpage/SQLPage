SELECT 'shell' as component,
    title,
    '/' as link,
    'index' as menu_item,
    'book' as icon
FROM spatial_data
WHERE id = $id;

SELECT 'datagrid' as component, title FROM spatial_data WHERE id = $id;

SELECT 'Latitude' as title,
    ST_Y(geom) as description,
    'purple' as color,
    'world-latitude' as icon
FROM spatial_data WHERE id = $id;

SELECT 'Longitude' as title,
    ST_X(geom) as description,
    'purple' as color,
    'world-longitude' as icon
FROM spatial_data WHERE id = $id;

SELECT 'Created at' as title,
    created_at as description,
    'calendar' as icon,
    'Date and time of creation' as footer
FROM spatial_data WHERE id = $id;

SELECT 'Label' as title,
    title as description,
    'geo:' || ST_Y(geom) || ',' || ST_X(geom) || '?z=16' AS link,
    'blue' as color,
    'world' as icon,
    'User-generated point name' as footer
FROM spatial_data
WHERE id = $id;

SELECT 'text' as component,
    description as contents_md
FROM spatial_data
WHERE id = $id;

SELECT 'list' as component, 'Closest points' as title;
SELECT to_label as title,
     ROUND(CvtToKm(distance), 3) || ' km' as description,
     'point.sql?id=' || to_id as link,
     'red' as color,
     'world-pin' as icon
FROM distances
WHERE from_id = $id
ORDER BY distance
LIMIT 5;

SELECT 'map' AS component,
        ST_Y(geom) AS latitude, ST_X(geom) AS longitude
FROM spatial_data WHERE id = $id;