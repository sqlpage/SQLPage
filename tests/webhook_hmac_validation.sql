-- Webhook HMAC signature validation example
-- This simulates receiving a webhook with HMAC signature in header

-- Redirect to error page if signature is missing
SELECT 'redirect' as component,
    '/error.sql?message=' || sqlpage.url_encode('Missing webhook signature') as link
WHERE sqlpage.header('X-Webhook-Signature') IS NULL;

-- Redirect to error page if signature is invalid
SELECT 'redirect' as component,
    '/error.sql?message=' || sqlpage.url_encode('Invalid webhook signature') as link
WHERE sqlpage.hmac(
    sqlpage.request_body(),
    sqlpage.environment_variable('WEBHOOK_SECRET'),
    'sha256-base64'
) != sqlpage.header('X-Webhook-Signature');

-- If we reach here, signature is valid - return success
SELECT 'json' as component;
SELECT json_object(
    'status', 'success',
    'message', 'Webhook signature verified',
    'body', sqlpage.request_body()
) as contents;
