SELECT 'text' as component,
    CASE 
        WHEN sqlpage.hmac('The quick brown fox jumps over the lazy dog', 'key', 'sha256') = 'f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8'
        THEN 'It works ! HMAC SHA-256 hash is correct'
        ELSE 'Hash mismatch: ' || sqlpage.hmac('The quick brown fox jumps over the lazy dog', 'key', 'sha256')
    END as contents;