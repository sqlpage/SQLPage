SELECT 'form' AS component, 'existing-form' AS id, 'Existing form' AS title, TRUE AS auto_submit;
SELECT 'existing_value' AS name, 'Existing value' AS label;
SELECT 'existing_file' AS name, 'Existing file' AS label, 'file' AS type;

SELECT 'card' AS component;
SELECT 'First fragment' AS title, '/fragment-loaded/widgets.sql?instance=first' AS embed;
SELECT 'Second fragment' AS title, '/fragment-loaded/widgets.sql?instance=second' AS embed;
