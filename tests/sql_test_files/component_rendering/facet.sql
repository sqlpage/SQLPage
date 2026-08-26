SELECT 'facet' AS component, 'Categories' AS description, '/all' AS all_link, 'All categories' AS all_title, 0 AS all_active;
SELECT 'It works !' AS title, '/active' AS link, 1 AS active;
SELECT 'Other category' AS title, '/other' AS link, 0 AS active;

SELECT 'facet' AS component, 'Categories' AS description, 1 AS compact, '/all' AS all_link, 1 AS all_active;
SELECT 'It works !' AS title, '/active' AS link, 0 AS active;
