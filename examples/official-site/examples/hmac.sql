select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, '

# HMAC (Hash-based Message Authentication Code)

In SQLPage, you can use the [`sqlpage.hmac`](/functions.sql?function=hmac) function to
compute a cryptographic signature for your data. HMAC is a type of message authentication code
that uses a cryptographic hash function and a secret key to verify both the data integrity
and authenticity of a message.

HMAC is commonly used for:
 - **API authentication**: Generate secure signatures for API requests
 - **Webhook verification**: Verify that webhook requests are authentic and haven''t been tampered with
 - **Token generation**: Create secure tokens for downloads, password resets, or temporary access
 - **Data integrity**: Ensure data hasn''t been modified during transmission or storage

The `sqlpage.hmac` function takes three parameters:
1. **data**: The message or data to be signed
2. **key**: A secret key that should be kept confidential
3. **algorithm**: The hash algorithm to use (optional, defaults to `sha256`)

The function returns a hexadecimal string that represents the HMAC signature.

## Security Notes

 - Keep your secret keys secure and never expose them in client-side code or version control
 - Use environment variables to store secret keys: `sqlpage.environment_variable(''SECRET_KEY'')`
 - For maximum security, use a key that is at least as long as the hash output
 - HMAC is deterministic: the same data and key will always produce the same signature

## Example Use Cases

### 1. API Request Signing

Generate a signature for API requests to verify they haven''t been tampered with:

```sql
SELECT sqlpage.hmac(
    ''user_id=123&action=update&timestamp='' || strftime(''%s'', ''now''),
    sqlpage.environment_variable(''API_SECRET''),
    ''sha256''
) as request_signature;
```

### 2. Webhook Signature Verification

Verify that a webhook request is authentic by comparing signatures:

```sql
SELECT 
    CASE 
        WHEN sqlpage.hmac(sqlpage.request_body(), ''webhook-secret'', ''sha256'') = :signature
        THEN ''Valid webhook''
        ELSE ''Invalid signature - request rejected''
    END as status;
```

### 3. Secure Download Tokens

Create time-limited download tokens that can be verified without storing them:

```sql
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

# Try it out

You can try the HMAC function out by entering data, a secret key, and selecting an algorithm below.
' as contents_md;

select 'form' as component, 'Generate HMAC' as validate;
select 'data' as name, 'text' as type, 'Data to Sign' as label, 
       'Enter the data you want to sign' as placeholder,
       coalesce(:data, 'Hello, World!') as value;
select 'key' as name, 'text' as type, 'Secret Key' as label,
       'Enter your secret key' as placeholder,
       coalesce(:key, 'my-secret-key') as value;
select 'algorithm' as name, 'select' as type, 'Hash Algorithm' as label;
select 'sha256' as value, :algorithm = 'sha256' or :algorithm is null as selected;
select 'sha512' as value, :algorithm = 'sha512' as selected;

select 'text' as component, '

### HMAC Signature

The HMAC signature for your data is:

```
' || sqlpage.hmac(:data, :key, :algorithm) || '
```

### Verification

To verify data later, simply recompute the HMAC with the same key and algorithm,
then compare the result with the original signature. If they match, the data is authentic and unmodified.

### Example Verification Code

```sql
SELECT 
    CASE 
        WHEN sqlpage.hmac(:received_data, :secret_key, :algorithm) = :stored_signature
        THEN ''Data is authentic''
        ELSE ''Data has been tampered with''
    END as verification_result;
```
' as contents_md
where :data is not null and :key is not null;