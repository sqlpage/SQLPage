-- Redirect if hash doesn't match expected value
SELECT 'redirect' as component, '/error.sql' as link
WHERE sqlpage.hmac('The quick brown fox jumps over the lazy dog', 'key', 'sha256') != 'f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8';

SELECT 'text' as component, 'It works ! HMAC SHA-256 hash is correct' as contents;