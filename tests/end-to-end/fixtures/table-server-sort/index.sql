SELECT 'table' AS component,
    'name' AS server_sort_column;

SELECT name
FROM (
    SELECT 'Zulu' AS name
    UNION ALL SELECT 'Alpha'
    UNION ALL SELECT 'Mike'
)
ORDER BY
    CASE WHEN $sort_name = 'ASCENDING' THEN name END ASC,
    CASE WHEN $sort_name = 'DESCENDING' THEN name END DESC;
