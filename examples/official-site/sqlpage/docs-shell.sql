-- Only the calling page defines the navigation and metadata.
SET options = sqlpage.variables('set');
SET section = $options->>'section';
SET catalog = CASE WHEN $section IN ('Components', 'Functions') AND $options->>'current' IS NULL THEN 1 END;

SELECT 'shell-docs' AS component,
    $options->>'title' AS title,
    $options->>'description' AS description,
    $section AS section,
    $catalog AS catalog,
    CASE WHEN $section IN ('Components', 'Functions') THEN lower($section) ELSE 'guides' END AS topic_label,
    CASE $section WHEN 'Components' THEN '/components.sql' WHEN 'Functions' THEN '/functions.sql' END AS catalog_link,
    $options->>'current' AS current_item,
    nullif($options->>'outline', '[]') AS outline,
    CASE
        WHEN $catalog THEN NULL
        WHEN $section = 'Components' THEN (
            SELECT json_group_array(json_object(
                'title', name, 'link', '/component.sql?component=' || name
            ) ORDER BY name) FROM component
        )
        WHEN $section = 'Functions' THEN (
            SELECT json_group_array(json_object(
                'title', name, 'link', '/functions.sql?function=' || name
            ) ORDER BY name) FROM sqlpage_functions
        )
        ELSE (
            SELECT json_group_array(json_object(
                'title', title, 'link', link
            ) ORDER BY sort_order) FROM guide
        )
    END AS topics;
