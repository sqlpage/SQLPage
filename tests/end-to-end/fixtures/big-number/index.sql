SELECT 'shell' AS component, COALESCE($theme, 'light') AS theme;

SELECT
    'big_number' AS component,
    2 AS columns;

SELECT
    'plain-color' AS id,
    'Plain value' AS title,
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
