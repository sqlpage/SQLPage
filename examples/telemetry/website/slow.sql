SELECT pg_sleep(15);

SELECT 'list' AS component,
       'Slow Query Complete' AS title;

SELECT 'The slow query finished successfully.' AS title,
       'This page exists to make PostgreSQL query-sample events easy to capture.' AS description;
