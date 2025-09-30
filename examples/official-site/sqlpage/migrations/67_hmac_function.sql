-- HMAC function documentation and examples

INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'hmac',
        '0.38.0',
        'shield-lock',
        'Creates a unique "signature" for your data using a secret key. This signature proves that the data hasn''t been tampered with and comes from someone who knows the secret.

Think of it like a wax seal on a letter - only someone with the right seal (your secret key) can create it, and if someone changes the letter, the seal won''t match anymore.

### What is HMAC used for?

**HMAC** (Hash-based Message Authentication Code) is commonly used to:
 - **Verify webhooks**: Check that notifications from services like Shopify, Stripe, or GitHub are genuine
 - **Secure API requests**: Prove that an API request comes from an authorized source
 - **Generate secure tokens**: Create temporary access codes for downloads or password resets
 - **Protect data**: Ensure data hasn''t been modified during transmission

### How to use it

The `sqlpage.hmac` function takes three inputs:
1. **Your data** - The text you want to sign (like a message or request body)
2. **Your secret key** - A password only you know (keep this safe!)
3. **Algorithm** (optional) - Either `sha256` (default) or `sha512`

It returns a long string of letters and numbers (the signature). If someone changes even one letter in your data, the signature will be completely different.

### Example 1: Verify Shopify Webhooks

When Shopify sends you a webhook (like when someone places an order), it includes a signature. Here''s how to verify it''s really from Shopify:

```sql
-- Shopify includes the signature in the X-Shopify-Hmac-SHA256 header
-- and sends the webhook data in the request body

SELECT ''text'' as component,
  CASE 
    WHEN sqlpage.hmac(
           sqlpage.request_body(),
           sqlpage.environment_variable(''SHOPIFY_WEBHOOK_SECRET''),
           ''sha256''
         ) = sqlpage.header(''X-Shopify-Hmac-SHA256'')
    THEN ''✅ Webhook verified! This is really from Shopify.''
    ELSE ''❌ Invalid signature - this might be fake!''
  END as contents;

-- If verified, process the order:
INSERT INTO orders (order_data, received_at)
SELECT 
  sqlpage.request_body(),
  datetime(''now'')
WHERE sqlpage.hmac(
        sqlpage.request_body(),
        sqlpage.environment_variable(''SHOPIFY_WEBHOOK_SECRET''),
        ''sha256''
      ) = sqlpage.header(''X-Shopify-Hmac-SHA256'');
```

### Example 2: Create Secure Download Links

Generate a token that expires after 1 hour:

```sql
-- Create a download token
INSERT INTO download_tokens (file_id, token, expires_at)
VALUES (
    :file_id,
    sqlpage.hmac(
        :file_id || ''|'' || datetime(''now'', ''+1 hour''),
        sqlpage.environment_variable(''DOWNLOAD_SECRET''),
        ''sha256''
    ),
    datetime(''now'', ''+1 hour'')
);
```

### Example 3: Sign API Requests

Prove your API request is authentic:

```sql
-- Create a signature for your API call
SELECT sqlpage.hmac(
    ''user_id=123&action=update&timestamp='' || strftime(''%s'', ''now''),
    ''my-secret-api-key'',
    ''sha256''
) as api_signature;
```

### Important Security Tips

 - **Keep your secret key safe**: Store it in environment variables using `sqlpage.environment_variable()`, never hardcode it in your SQL files
 - **Use strong keys**: Your secret should be long and random (at least 32 characters)
 - **The signature is case-sensitive**: Even one wrong letter means the signature won''t match
 - **Algorithms**: Use `sha256` for most cases (it''s the default), or `sha512` for extra security
 - **NULL handling**: If your data or key is NULL, the function returns NULL
'
    );

INSERT INTO sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES (
        'hmac',
        1,
        'data',
        'The input data to compute the HMAC for. Can be any text string.',
        'TEXT'
    ),
    (
        'hmac',
        2,
        'key',
        'The secret key used to compute the HMAC. Should be kept confidential.',
        'TEXT'
    ),
    (
        'hmac',
        3,
        'algorithm',
        'The hash algorithm to use. Optional, defaults to `sha256`. Supported values: `sha256`, `sha512`.',
        'TEXT'
    );