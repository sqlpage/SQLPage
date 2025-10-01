-- Webhook HMAC signature validation example
-- This simulates receiving a webhook with HMAC signature in header
-- Redirect to error page if signature is invalid
-- test this with: curl localhost:8080/tests/webhook_hmac_validation.sql -H 'X-Webhook-Signature: 260b3b5ead84843645588af82d5d2c3fe24c598a950d36c45438c3a5f5bb941c' -H 'Content-Type: application/json' --data-raw '{"order_id":12345,"total":"99.99"}' -v
SET body = sqlpage.request_body();
SET secret = sqlpage.environment_variable('WEBHOOK_SECRET');
SET expected_signature = sqlpage.hmac($body, $secret, 'sha256');
SET actual_signature =  sqlpage.header('X-Webhook-Signature');

SELECT
    'redirect' as component,
    '/error.sql?err=bad_webhook_signature' as link
WHERE $actual_signature != $expected_signature OR $actual_signature IS NULL;

-- If we reach here, signature is valid - return success
SELECT 'json' as component, 'jsonlines' as type;
select 'Webhook signature is valid !' as msg;