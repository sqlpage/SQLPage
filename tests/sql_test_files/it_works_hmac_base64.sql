-- Test HMAC with base64 output format
-- Redirect if hash doesn't match expected value
SELECT 'redirect' as component, '/error.sql' as link
WHERE sqlpage.hmac('The quick brown fox jumps over the lazy dog', 'key', 'sha256-base64') != '97yD9DBThCSxMpjmqm+xQ+9NWaFJRhdZl0edvC0aPNg=';

SELECT 'text' as component, 'It works ! HMAC SHA-256 base64 output is correct' as contents;