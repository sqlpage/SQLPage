SELECT 'dynamic' AS component,
    sqlpage.run_sql('form/index.sql', '{"fragment":"1","id":"existing-form"}') AS properties;
SELECT 'card' AS component;
SELECT 'Embedded form' AS title, '/form/?fragment=1&id=first-form' AS embed;
SELECT 'Embedded chart' AS title, '/chart/' AS embed;
