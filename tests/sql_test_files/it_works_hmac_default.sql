SELECT 'text' as component;
SELECT 'HMAC (default sha256): ' || sqlpage.hmac('Hello, World!', 'secret-key') as contents;