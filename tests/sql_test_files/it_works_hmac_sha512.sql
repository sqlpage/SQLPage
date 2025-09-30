SELECT 'text' as component,
    CASE 
        WHEN sqlpage.hmac('The quick brown fox jumps over the lazy dog', 'key', 'sha512') = 'b42af09057bac1e2d41708e48a902e09b5ff7f12ab428a4fe86653c73dd248fb82f948a549f7b791a5b41915ee4d1ec3935357e4e2317250d0372afa2ebeeb3a'
        THEN 'It works ! HMAC SHA-512 hash is correct'
        ELSE 'Hash mismatch: ' || sqlpage.hmac('The quick brown fox jumps over the lazy dog', 'key', 'sha512')
    END as contents;