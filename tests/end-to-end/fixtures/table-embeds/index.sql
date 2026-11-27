SELECT 'table' AS component, 'table-embeds' AS id, TRUE AS sort, TRUE AS search,
       TRUE AS freeze_headers, TRUE AS freeze_columns,
       'preview' AS embed, 'Details' AS embed;

SELECT 'Moss' AS "Name", 'sample.sql?count=3&label=Inner-only' AS "Preview",
       '' AS "Details", 10 AS "Rank"
UNION ALL
SELECT 'Pine', 'sample.sql?count=1&label=Second', '/form/?fragment=1&id=cell-form', 2
UNION ALL
SELECT 'Oak', NULL, NULL, 1000;
