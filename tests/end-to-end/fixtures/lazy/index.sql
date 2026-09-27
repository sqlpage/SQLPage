SELECT 'lazy' AS component;
SELECT '/lazy/content.sql' AS embed, 'height:200px' AS style;

SELECT 'card' AS component;
SELECT 'Embedded card' AS title, '/lazy/card.sql' AS embed;
SELECT 'Iframe card' AS title, '/lazy/frame.sql' AS embed, 'iframe' AS embed_mode;
