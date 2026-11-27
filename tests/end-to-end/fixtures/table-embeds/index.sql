SELECT 'table' AS component, 'preview' AS embed, 'Details' AS embed;

SELECT 'Moss' AS "Name", 'sample.sql?label=Forest%20preview' AS "Preview",
       'sample.sql?label=Grows%20in%20shade' AS "Details"
UNION ALL
SELECT 'Pine', NULL, '';
