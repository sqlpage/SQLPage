-- Redirect if default algorithm doesn't match sha256
SELECT 'redirect' as component, '/error.sql' as link
WHERE sqlpage.hmac('test data', 'test key') != sqlpage.hmac('test data', 'test key', 'sha256');

SELECT 'text' as component, 'It works ! HMAC default algorithm is SHA-256' as contents;