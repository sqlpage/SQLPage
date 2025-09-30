-- Test Shopify webhook HMAC validation
-- Shopify sends webhook body and HMAC signature in X-Shopify-Hmac-SHA256 header

SELECT 'text' as component,
    CASE 
        -- Example webhook data and signature (simulating Shopify webhook)
        WHEN sqlpage.hmac(
            '{"id":1234567890,"email":"customer@example.com","total_price":"123.45"}',
            'test-webhook-secret',
            'sha256'
        ) = '40dc8e6d394a6ccc76a8394f17f64e65c06a8393a03e0fb6a24cb7ce575cd06c'
        THEN 'It works ! Shopify webhook signature verified'
        ELSE 'Signature mismatch: ' || sqlpage.hmac('{"id":1234567890,"email":"customer@example.com","total_price":"123.45"}', 'test-webhook-secret', 'sha256')
    END as contents;