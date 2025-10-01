-- HMAC function documentation and examples
INSERT INTO
    sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES
    (
        'hmac',
        '0.38.0',
        'shield-lock',
        'Creates a unique "signature" for some data using a secret key.
This signature proves that the data hasn''t been tampered with and comes from someone who knows the secret.

### What is HMAC used for?

[**HMAC**](https://en.wikipedia.org/wiki/HMAC) (Hash-based Message Authentication Code) is commonly used to:
 - **Verify webhooks**: Use HMAC to ensure only a given external service can call a given endpoint in your application.
The service signs their request with a secret key, and you verify the signature before processing the data they sent you.
Used for instance by [Stripe](https://docs.stripe.com/webhooks?verify=verify-manually), and [Shopify](https://shopify.dev/docs/apps/build/webhooks/subscribe/https#step-2-validate-the-origin-of-your-webhook-to-ensure-its-coming-from-shopify).
 - **Secure API requests**: Prove that an API request comes from an authorized source
 - **Generate secure tokens**: Create temporary access codes for downloads or password resets
 - **Protect data**: Ensure data hasn''t been modified during transmission

### How to use it

The `sqlpage.hmac` function takes three inputs:
1. **Your data** - The text you want to sign (like a message or request body)
2. **Your secret key** - A password only you know (keep this safe!)
3. **Algorithm** (optional) - The hash algorithm and output format:
   - `sha256` (default) - SHA-256 with hexadecimal output
   - `sha256-base64` - SHA-256 with base64 output
   - `sha512` - SHA-512 with hexadecimal output
   - `sha512-base64` - SHA-512 with base64 output

It returns a signature string. If someone changes even one letter in your data, the signature will be completely different.

### Example: Verify a Webhooks signature

When Shopify sends you a webhook (like when someone places an order), it includes a signature. Here''s how to verify it''s really from Shopify.
This supposes you store the secret key in an [environment variable](https://en.wikipedia.org/wiki/Environment_variable) named `WEBHOOK_SECRET`.

```sql
SET body = sqlpage.request_body();
SET secret = sqlpage.environment_variable(''WEBHOOK_SECRET'');
SET expected_signature = sqlpage.hmac($body, $secret, ''sha256'');
SET actual_signature =  sqlpage.header(''X-Webhook-Signature'');

-- redirect to an error page and stop execution if the signature does not match
SELECT
    ''redirect'' as component,
    ''/error.sql?err=bad_webhook_signature'' as link
WHERE $actual_signature != $expected_signature OR $actual_signature IS NULL;

-- If we reach here, the signature is valid - process the order
INSERT INTO orders (order_data) VALUES ($body);

SELECT ''json'' as component, ''jsonlines'' as type;
SELECT ''success'' as status;
```

### Example: Time-limited links

You can create links that will be valid only for a limited time by including a signature in them.
Let''s say we have a `download.sql` page we want to link to,
but we don''t want it to be accessible to anyone who can find the link.
Sign `file_id|expires_at` with a secret. Accept only if not expired and the signature matches.

#### Generate a signed link

```sql
SET expires_at = datetime(''now'', ''+1 hour'');
SET token = sqlpage.hmac(
    $file_id || ''|'' || $expires_at,
    sqlpage.environment_variable(''DOWNLOAD_SECRET''),
    ''sha256''
);
SELECT ''/download.sql?file_id='' || $file_id || ''&expires_at='' || $expires_at || ''&token='' || $token AS link;
```

#### Verify the signed link

```sql
SET expected = sqlpage.hmac(
    $file_id || ''|'' || $expires_at,
    sqlpage.environment_variable(''DOWNLOAD_SECRET''),
    ''sha256''
);
SELECT ''redirect'' AS component, ''/error.sql?err=expired'' AS link
WHERE $expected != $token OR $token IS NULL OR $expires_at < datetime(''now'');

-- serve the file
```

### Important Security Notes

 - **Keep your secret key safe**: If your secret leaks, anyone can forge signatures and access protected pages
 - **The signature is case-sensitive**: Even a single wrong letter means the signature won''t match
 - **NULL handling**: Always use `IS DISTINCT FROM`, not `=` to check for hmac matches.
   - `SELECT ''redirect'' as component WHERE sqlpage.hmac(...) != $signature` will not redirect if `$signature` is NULL (the signature is absent).
   - `SELECT ''redirect'' as component WHERE sqlpage.hmac(...) IS DISTINCT FROM $signature` checks for both NULL and non-NULL values (but is not available in all SQL dialects).
   - `SELECT ''redirect'' as component WHERE sqlpage.hmac(...) != $signature OR $signature IS NULL` is the most portable solution.
'
    );

INSERT INTO
    sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES
    (
        'hmac',
        1,
        'data',
        'The input data to compute the HMAC for. Can be any text string. Cannot be NULL.',
        'TEXT'
    ),
    (
        'hmac',
        2,
        'key',
        'The secret key used to compute the HMAC. Should be kept confidential. Cannot be NULL.',
        'TEXT'
    ),
    (
        'hmac',
        3,
        'algorithm',
        'The hash algorithm and output format. Optional, defaults to `sha256` (hex output). Supported values: `sha256`, `sha256-base64`, `sha512`, `sha512-base64`. Defaults to `sha256`.',
        'TEXT'
    );