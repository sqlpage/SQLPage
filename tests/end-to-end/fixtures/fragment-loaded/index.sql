SELECT 'dynamic' AS component,
    sqlpage.run_sql('form/index.sql', '{"fragment":"1","id":"existing-form"}') AS properties WHERE $lazy IS NULL;
SELECT 'card' AS component WHERE $lazy IS NULL;
SELECT 'Embedded form' AS title, '/form/?fragment=1&id=first-form' AS embed WHERE $lazy IS NULL;
SELECT 'Embedded chart' AS title, '/chart/' AS embed WHERE $lazy IS NULL;
SELECT 'text' AS component, 'Fragment lifecycle' AS contents WHERE $lazy IS NOT NULL;
