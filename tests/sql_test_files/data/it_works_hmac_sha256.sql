select 'f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8' as expected,
    sqlpage.hmac('The quick brown fox jumps over the lazy dog', 'key', 'sha256') as actual;