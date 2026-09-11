-- Exercise every form layout that renders a field description:
-- standard controls, the legacy checkbox/radio controls, and switches.
SELECT
    'form' AS component,
    'Form description markdown' AS title,
    '' AS validate;

SELECT
    'modern_text' AS name,
    'Modern text field' AS label,
    '**Bold text** and *italic text*.' AS description_md;

SELECT
    'modern_textarea' AS name,
    'Modern textarea' AS label,
    'textarea' AS type,
    '**Bold textarea** and *italic textarea*.' AS description_md;

SELECT
    'modern_select' AS name,
    'Modern select' AS label,
    'select' AS type,
    '[{"label":"Option","value":"option"}]' AS options,
    '**Bold select** and *italic select*.' AS description_md;

SELECT
    'legacy_radio' AS name,
    'Legacy radio' AS label,
    'radio' AS type,
    'radio' AS value,
    '**Bold radio** and *italic radio*.' AS description_md;

SELECT
    'legacy_checkbox' AS name,
    'Legacy checkbox' AS label,
    'checkbox' AS type,
    'checkbox' AS value,
    '**Bold checkbox** and *italic checkbox*.' AS description_md;

SELECT
    'modern_switch' AS name,
    'Modern switch' AS label,
    'switch' AS type,
    'switch' AS value,
    '**Bold switch** and *italic switch*.' AS description_md;
