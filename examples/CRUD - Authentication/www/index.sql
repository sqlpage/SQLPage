SELECT
   'dynamic' AS component,
    json_array(
        json_object(
            'component',      'shell',
            'title',          'CRUD with Authentication',
            'icon',           'database',
            'description',    'Description',

            'css',
                json_array(
                    '/css/prism-tabler-theme.css'
                ),

            'javascript',
                json_array(
                    'https://cdn.jsdelivr.net/npm/prismjs@1/components/prism-core.min.js',
                    'https://cdn.jsdelivr.net/npm/prismjs@1/plugins/autoloader/prism-autoloader.min.js'
                )

        )
    ) AS properties;

-- =============================================================================

SELECT
    'text'  AS component,
    TRUE    AS center,
    2       AS level,
    'Demo/Template CRUD with Authentication' AS title,
    sqlpage.read_file_as_text('./README.md') AS contents_md;



