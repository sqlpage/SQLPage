select sqlpage.hmac('test data', 'test key', 'sha256') as expected,
    sqlpage.hmac('test data', 'test key') as actual;