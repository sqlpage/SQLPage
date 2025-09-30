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
        'Computes the [HMAC](https://en.wikipedia.org/wiki/HMAC) (Hash-based Message Authentication Code) of the input data using a secret key and a cryptographic hash function.

HMAC is used to verify both the data integrity and authenticity of a message. It is commonly used for:
 - Generating secure tokens and signatures
 - API request authentication
 - Webhook signature verification
 - Data integrity validation

### Example

#### Generate an HMAC for API authentication

```sql
-- Generate a secure signature for an API request
SELECT sqlpage.hmac(
    ''user_id=123&action=update'',
    ''my-secret-api-key'',
    ''sha256''
) as request_signature;
```

#### Verify a webhook signature

```sql
-- Verify that a webhook request is authentic
SELECT 
    CASE 
        WHEN sqlpage.hmac(sqlpage.request_body(), ''webhook-secret'', ''sha256'') = :signature
        THEN ''Valid webhook''
        ELSE ''Invalid signature''
    END as status;
```

#### Create a secure download token

```sql
-- Generate a time-limited download token
INSERT INTO download_tokens (file_id, token, expires_at)
VALUES (
    :file_id,
    sqlpage.hmac(
        :file_id || ''|'' || datetime(''now'', ''+1 hour''),
        sqlpage.environment_variable(''SECRET_KEY''),
        ''sha256''
    ),
    datetime(''now'', ''+1 hour'')
);
```

### Notes

 - The function returns a hexadecimal string representation of the HMAC.
 - If either `data` or `key` is NULL, the function returns NULL.
 - The `algorithm` parameter is optional and defaults to `sha256` if not specified.
 - Supported algorithms: `sha256`, `sha512`.
 - The key can be of any length. For maximum security, use a key that is at least as long as the hash output (32 bytes for SHA-256, 64 bytes for SHA-512).
 - Keep your secret keys secure and never expose them in client-side code or version control.
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