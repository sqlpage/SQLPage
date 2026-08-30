SELECT
    'dynamic' AS component,
    COALESCE(
        $properties,
        '[{"component":"map","title":"Map test fixture","tile_source":false}]'
    ) AS properties;
