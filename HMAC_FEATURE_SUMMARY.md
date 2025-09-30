# SQLPage HMAC Function - Feature Summary

## Overview
This document summarizes the addition of the `sqlpage.hmac()` function to SQLPage, which provides cryptographic HMAC (Hash-based Message Authentication Code) capabilities.

## Changes Made

### 1. Dependencies Added (Cargo.toml)
- `hmac = "0.12"` - HMAC implementation from RustCrypto
- `sha2 = "0.10"` - SHA-2 family of hash functions

### 2. Function Implementation (src/webserver/database/sqlpage_functions/functions.rs)

#### Function Declaration
```rust
hmac(data: Option<Cow<str>>, key: Option<Cow<str>>, algorithm: Option<Cow<str>>);
```

#### Implementation Features
- **Parameters:**
  - `data`: The input data to compute HMAC for
  - `key`: The secret key used for HMAC
  - `algorithm`: Optional hash algorithm (defaults to "sha256")

- **Supported Algorithms:**
  - `sha256` (default)
  - `sha512`

- **Return Value:**
  - Hexadecimal string representation of the HMAC
  - Returns NULL if either data or key is NULL

- **Security:**
  - Uses industry-standard RustCrypto libraries
  - Constant-time comparison available via HMAC library
  - Proper error handling for invalid inputs

### 3. Documentation (examples/official-site/sqlpage/migrations/08_functions.sql)

Added comprehensive documentation including:
- Function description with Wikipedia link
- Common use cases (API authentication, webhook verification, token generation)
- Three detailed code examples:
  1. API request signing
  2. Webhook signature verification
  3. Secure download tokens
- Security notes and best practices
- Parameter descriptions

### 4. Interactive Example (examples/official-site/examples/hmac.sql)

Created a full interactive example page that allows users to:
- Try the HMAC function with custom data and keys
- Select different hash algorithms
- See the resulting signature
- Learn about verification techniques
- Understand real-world use cases

### 5. Test Suite (tests/sql_test_files/)

Added four test files:
- `it_works_hmac_sha256.sql` - Test SHA-256 algorithm
- `it_works_hmac_sha512.sql` - Test SHA-512 algorithm
- `it_works_hmac_default.sql` - Test default algorithm behavior
- `it_works_hmac_null.sql` - Test NULL handling

## Usage Examples

### Basic Usage
```sql
SELECT sqlpage.hmac('Hello, World!', 'secret-key', 'sha256') as signature;
```

### API Request Signing
```sql
SELECT sqlpage.hmac(
    'user_id=123&action=update',
    'my-secret-api-key',
    'sha256'
) as request_signature;
```

### Webhook Verification
```sql
SELECT 
    CASE 
        WHEN sqlpage.hmac(sqlpage.request_body(), 'webhook-secret', 'sha256') = :signature
        THEN 'Valid webhook'
        ELSE 'Invalid signature'
    END as status;
```

### Secure Token Generation
```sql
INSERT INTO download_tokens (file_id, token, expires_at)
VALUES (
    :file_id,
    sqlpage.hmac(
        :file_id || '|' || datetime('now', '+1 hour'),
        sqlpage.environment_variable('SECRET_KEY'),
        'sha256'
    ),
    datetime('now', '+1 hour')
);
```

## Security Considerations

1. **Key Management:**
   - Keys should be stored securely using environment variables
   - Never hardcode keys in SQL files or expose them client-side
   - Use `sqlpage.environment_variable()` to access keys

2. **Algorithm Selection:**
   - SHA-256 is the default and suitable for most use cases
   - SHA-512 provides additional security for highly sensitive applications
   - Both algorithms are cryptographically secure

3. **Key Length:**
   - Keys can be of any length
   - For maximum security, use keys at least as long as the hash output:
     - SHA-256: 32 bytes (64 hex characters)
     - SHA-512: 64 bytes (128 hex characters)

## Testing

To run the tests:
```bash
cargo test
```

The test suite includes:
- Algorithm verification (SHA-256, SHA-512)
- Default parameter handling
- NULL value handling
- Integration with SQLPage's query execution

## Documentation Links

Once deployed, the function will be documented at:
- Function reference: `/functions.sql?function=hmac`
- Interactive example: `/examples/hmac.sql`

## Version

Introduced in SQLPage version **0.38.0**

## Files Modified

1. `/workspace/Cargo.toml` - Added dependencies
2. `/workspace/src/webserver/database/sqlpage_functions/functions.rs` - Implementation
3. `/workspace/examples/official-site/sqlpage/migrations/08_functions.sql` - Documentation
4. `/workspace/examples/official-site/examples/hmac.sql` - Interactive example
5. `/workspace/tests/sql_test_files/it_works_hmac_*.sql` - Test suite

## Related Functions

- `sqlpage.hash_password()` - Password hashing using Argon2
- `sqlpage.random_string()` - Generate cryptographically secure random strings
- `sqlpage.environment_variable()` - Access environment variables for secrets