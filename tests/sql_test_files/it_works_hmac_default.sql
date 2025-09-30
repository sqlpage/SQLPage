SELECT 'text' as component,
    CASE 
        WHEN sqlpage.hmac('test data', 'test key') = sqlpage.hmac('test data', 'test key', 'sha256')
        THEN 'It works ! HMAC default algorithm is SHA-256'
        ELSE 'Default algorithm mismatch'
    END as contents;