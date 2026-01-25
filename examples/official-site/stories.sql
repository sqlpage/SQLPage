SELECT 'http_header' AS component, 
    'no-store, no-cache, must-revalidate, max-age=0' AS "Cache-Control",
    'no-cache' AS "pragma",
    '0' AS "expires",
    '<https://sql-page.com/safety>; rel="canonical"' AS "Link";

SELECT 'dynamic' AS component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', 'SQLPage Success Stories'
)) AS properties
FROM example WHERE component = 'shell' LIMIT 1;

SET TEXT_MAX_LENGTH = 300;

SELECT 
    'alert'                    AS component,
    CONCAT('You have selected the "', $filter, '" filter.') AS title,
    'filter'                   AS icon,
    'teal'                     AS color,
    TRUE                       AS dismissible,
    '[Click here to deactivate it.](stories)' AS description_md
WHERE $id IS NULL 
  AND $filter IS NOT NULL

SELECT
    'stories' AS component, 
    $filter AS filter,
    id,
    title,
    publication_date,
    tags,
    CASE
        WHEN LENGTH(contents_md) > CAST($EXT_MAX_LENGTH AS INTEGER) THEN SUBSTR(contents_md, 1, CAST($TEXT_MAX_LENGTH AS INTEGER)) || '...'
        ELSE contents_md
    END as truncated_contents
FROM stories
WHERE 
        $id IS NULL
  AND   ($filter IS NULL OR EXISTS (SELECT 1 FROM json_each(tags) WHERE value LIKE $filter COLLATE NOCASE))
ORDER BY publication_date DESC;

SELECT
    'story' AS component, 
    $filter AS filter,
    id,
    title,
    publication_date,
    contents_md,
    optional_contents_md,
    image,
    website,
    git_repository
FROM stories
WHERE id = $id;
