SELECT 'form' AS component, 'root-form' AS id, 'Fragment form' AS title, TRUE AS auto_submit;
SELECT 'root_value' AS name, 'Root value' AS label;
SELECT 'root_file' AS name, 'Root file' AS label, 'file' AS type;
SELECT 'root_select' AS name, 'Root select' AS label, 'select' AS type, TRUE AS searchable,
    '[{"label":"One","value":"one"},{"label":"Two","value":"two"}]' AS options;
