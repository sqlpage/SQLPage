SELECT 'shell' as component,
    title,
    '/' as link,
    'index' as menu_item,
    'book' as icon
FROM spatial_data
WHERE id = $id::int;

SELECT 'datagrid' as component, title FROM spatial_data WHERE id = $id::int;

SELECT 'Latitude' as title,
    ST_Y(geom) as description,
    'purple' as color,
    'world-latitude' as icon
FROM spatial_data WHERE id = $id::int;

SELECT 'Longitude' as title,
    ST_X(geom) as description,
    'purple' as color,
    'world-longitude' as icon
FROM spatial_data WHERE id = $id::int;

SELECT 'Created at' as title,
    created_at as description,
    'calendar' as icon,
    'Date and time of creation' as footer
FROM spatial_data WHERE id = $id::int;

SELECT 'Label' as title,
    title as description,
    'geo:' || ST_Y(geom) || ',' || ST_X(geom) || '?z=16' AS link,
    'blue' as color,
    'world' as icon,
    'User-generated point name' as footer
FROM spatial_data
WHERE id = $id::int;

SELECT 'text' as component,
    description || 
    format(E'\n\n [Edit description](edition_form.sql?id=%s)', id)  
    as contents_md
FROM spatial_data
WHERE id = $id::int;

SELECT 'list' as component, 'Closest points' as title;
SELECT to_label as title,
     ROUND(distance::decimal, 3) || ' km' as description,
     'point.sql?id=' || to_id as link,
     'red' as color,
     'world-pin' as icon
FROM distances
WHERE from_id = $id::int
ORDER BY distance
LIMIT 5;

SELECT 'map' AS component,
        ST_Y(geom) AS latitude, ST_X(geom) AS longitude
FROM spatial_data WHERE id = $id::int;