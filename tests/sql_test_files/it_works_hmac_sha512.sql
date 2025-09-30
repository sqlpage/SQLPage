SELECT 'text' as component;
SELECT 'HMAC SHA-512: ' || sqlpage.hmac('Hello, World!', 'secret-key', 'sha512') as contents;