SELECT 'text' as component;
SELECT 'HMAC SHA-256: ' || sqlpage.hmac('Hello, World!', 'secret-key', 'sha256') as contents;