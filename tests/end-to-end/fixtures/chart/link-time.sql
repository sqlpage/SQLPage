SELECT 'chart' AS component, 'test-chart' AS id, 'Chart test fixture' AS title,
    'line' AS type, TRUE AS time, 8 AS marker;
SELECT 'Points' AS series, '2024-03-01' AS x, 10 AS y, '/linked-before.sql' AS link;
SELECT 'Points' AS series, '2024-03-02' AS x, 15 AS y, '/linked.sql' AS link;
SELECT 'Points' AS series, '2024-03-03' AS x, 20 AS y, '/linked-after.sql' AS link;
