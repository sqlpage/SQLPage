SELECT 'text' as component;
SELECT 'HMAC with null data: ' || coalesce(sqlpage.hmac(NULL, 'secret-key', 'sha256'), 'NULL') as contents;
SELECT 'HMAC with null key: ' || coalesce(sqlpage.hmac('data', NULL, 'sha256'), 'NULL') as contents;