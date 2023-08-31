SELECT 'shell' as component,
    'Point edition' AS title,
    '/' as link,
    'book' as icon;

UPDATE spatial_data
SET description = :Text
WHERE id = $id::int AND :Text IS NOT NULL;

SELECT 'form' AS component, title
FROM spatial_data WHERE id = $id::int;

SELECT
    'Text' AS name,
    'Description for the point: ' || title AS description,
    'textarea' AS type,
    description AS value
FROM spatial_data
WHERE id = $id::int;

SELECT 'text' as component,
    description as contents_md
FROM spatial_data
WHERE id = $id::int;