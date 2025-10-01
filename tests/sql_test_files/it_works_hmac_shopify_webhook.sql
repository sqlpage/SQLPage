-- Test Shopify webhook HMAC validation with base64 output
-- Shopify sends webhook body and HMAC signature in X-Shopify-Hmac-SHA256 header (base64 format)

-- Redirect to error if signature doesn't match (proper pattern)
SELECT 'redirect' as component,
    '/error.sql?msg=invalid_signature' as link
WHERE sqlpage.hmac(
    '{"id":1234567890,"email":"customer@example.com","total_price":"123.45"}',
    'test-webhook-secret',
    'sha256-base64'
) != 'QNyObTlKbMx2qDlPF/ZOZcBqg5OgPg+2oky3zldc0Gw=';

-- If we reach here, signature is valid
SELECT 'text' as component,
    'It works ! Shopify webhook signature verified (base64 format)' as contents;