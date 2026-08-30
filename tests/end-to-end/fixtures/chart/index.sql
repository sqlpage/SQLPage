SELECT
    'dynamic' AS component,
    COALESCE(
        $properties,
        '[{"component":"chart","title":"Chart test fixture"},{"x":"Ready","y":1}]'
    ) AS properties;
