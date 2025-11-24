-- Test Shopify webhook HMAC validation with base64 output
select 'QNyObTlKbMx2qDlPF/ZOZcBqg5OgPg+2oky3zldc0Gw=' as expected,
    sqlpage.hmac(
        '{"id":1234567890,"email":"customer@example.com","total_price":"123.45"}',
        'test-webhook-secret',
        'sha256-base64'
    ) as actual;