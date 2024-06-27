-- =============================================================================
-- Displays GET/POST/SET variables in sorted tables for debug purposes. The
-- variables are displayed if URL GET variable &DEBUG=1 is set OR &error is
-- defined and not empty.
--
-- ## Usage
--
-- Execute this script at the bottom of a page script via
--
-- ```sql
-- SELECT
--     'dynamic' AS component,
--     sqlpage.run_sql('footer_debug_post-get-set.sql') AS properties;
-- ```

-- GET VARIABLES --

SELECT
    'title'          AS component,
    'GET Variables'  AS contents,
    3                AS level,
    TRUE             AS center
WHERE $DEBUG OR $error IS NOT NULL;

SELECT
    'table'          AS component,
    TRUE             AS sort,
    TRUE             AS search,
    TRUE             AS border,
    TRUE             AS hover,
    FALSE            AS striped_columns,
    TRUE             AS striped_rows,
    'value'          AS markdown
WHERE $DEBUG OR $error IS NOT NULL;

SELECT "key" AS variable, value
FROM json_each(sqlpage.variables('GET'))
WHERE $DEBUG OR $error IS NOT NULL
ORDER BY substr("key", 1, 1) = '_', "key";


-- POST VARIABLES --

SELECT
    'title'          AS component,
    'POST Variables' AS contents,
    3                AS level,
    TRUE             AS center
WHERE $DEBUG OR $error IS NOT NULL;

SELECT
    'table'          AS component,
    TRUE             AS sort,
    TRUE             AS search,
    TRUE             AS border,
    TRUE             AS hover,
    FALSE            AS striped_columns,
    TRUE             AS striped_rows,
    'value'          AS markdown
WHERE $DEBUG OR $error IS NOT NULL;

SELECT "key" AS variable, value
FROM json_each(sqlpage.variables('POST'))
WHERE $DEBUG OR $error IS NOT NULL
ORDER BY "key";
