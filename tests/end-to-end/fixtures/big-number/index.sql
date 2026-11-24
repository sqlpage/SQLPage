SELECT 'shell' AS component, COALESCE($theme, 'light') AS theme;

SELECT
    'big_number' AS component,
    2 AS columns;

SELECT
    'plain-color' AS id,
    'Plain value' AS title,
    '#plain-color' AS title_link,
    '1,234' AS value,
    'red' AS color;

SELECT
    'linked-color' AS id,
    'Linked value' AS title,
    '5,678' AS value,
    'blue' AS color,
    '#linked-color' AS value_link;

SELECT
    'default-color' AS id,
    'Default value' AS title,
    '9,012' AS value;

SELECT 'big_number' AS component, 2 AS columns, $wrap = 'true' AS wrap;

SELECT
    'long-text' AS id,
    'Items checked across all locations during the reporting period' AS title,
    120 AS value,
    'Includes items from the main collection and all storage locations.' AS description;

SELECT 'short-text' AS id, 'Locations' AS title, 8 AS value;
