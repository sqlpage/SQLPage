-- Repeated variables must use the same parameter in SELECT and GROUP BY.
-- https://github.com/sqlpage/SQLPage/issues/1558
SET b = '3';

SELECT '0:2,1:2,2:2' AS expected,
    STRING_AGG(CONCAT(bucket, ':', cnt), ',') WITHIN GROUP (ORDER BY bucket) AS actual
FROM (
    SELECT n % CAST($b AS int) AS bucket, COUNT(*) AS cnt
    FROM (VALUES (1), (2), (3), (4), (5), (6)) t(n)
    GROUP BY n % CAST($b AS int)
) buckets;
