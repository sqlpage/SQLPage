# OIDC Query Parameter Preservation Fix

## Problem
When using OIDC authentication in SQLPage, query parameters were lost during the authentication redirect flow. For example:
- User visits: `/page.sql?param=1`
- After OIDC authentication, user is redirected to: `/page.sql` (query parameters lost)

## Root Cause
In `src/webserver/oidc.rs`, the `OidcLoginState::new()` function was only capturing `request.path()` which excludes query parameters, instead of the full URL.

## Fix Applied

### 1. Modified `OidcLoginState::new()` method
**Before:**
```rust
fn new(request: &ServiceRequest, auth_url: AuthUrlParams) -> Self {
    Self {
        initial_url: request.path().to_string(), // BUG: loses query parameters
        csrf_token: auth_url.csrf_token,
        nonce: auth_url.nonce,
    }
}
```

**After:**
```rust
fn new(request: &ServiceRequest, auth_url: AuthUrlParams) -> Self {
    // Capture the full path with query string for proper redirect after auth
    let initial_url = Self::build_safe_redirect_url(request);
    
    Self {
        initial_url,
        csrf_token: auth_url.csrf_token,
        nonce: auth_url.nonce,
    }
}
```

### 2. Added `build_safe_redirect_url()` method
This method safely constructs the redirect URL by:
- Using `request.path()` to get the path
- Using `request.query_string()` to get query parameters
- Combining them while ensuring security (path must start with '/')

```rust
fn build_safe_redirect_url(request: &ServiceRequest) -> String {
    let path = request.path();
    let query = request.query_string();
    
    // Ensure the path starts with '/' for security (prevent open redirects)
    let safe_path = if path.starts_with('/') {
        path
    } else {
        "/"
    };
    
    if query.is_empty() {
        safe_path.to_string()
    } else {
        format!("{}?{}", safe_path, query)
    }
}
```

### 3. Added `validate_redirect_url()` function
Added additional security validation when retrieving the URL from the cookie:

```rust
fn validate_redirect_url(url: &str) -> String {
    // Only allow relative URLs that start with '/' to prevent open redirects
    if url.starts_with('/') && !url.starts_with("//") {
        url.to_string()
    } else {
        log::warn!("Invalid redirect URL '{}', redirecting to root instead", url);
        "/".to_string()
    }
}
```

### 4. Updated callback processing
Modified the OIDC callback processing to use the validation function:

**Before:**
```rust
let mut response = build_redirect_response(state.initial_url);
```

**After:**
```rust
// Validate the redirect URL is safe before using it
let redirect_url = validate_redirect_url(&state.initial_url);
let mut response = build_redirect_response(redirect_url);
```

## Security Considerations

The fix includes several security measures:

1. **Open Redirect Prevention**: Only relative URLs starting with '/' are allowed
2. **Protocol-relative URL Prevention**: URLs starting with '//' are rejected
3. **Absolute URL Prevention**: URLs with protocols (http://, https://) are rejected
4. **Fallback to Root**: Invalid URLs redirect to '/' instead of failing

## Testing

Unit tests were added to verify:
- Query parameters are preserved correctly
- Special characters in URLs are handled properly
- Security validations work as expected
- Invalid URLs are safely handled

Example test case:
```rust
#[test]
fn test_oidc_login_state_preserves_query_parameters() {
    let req = test::TestRequest::with_uri("/dashboard.sql?user_id=123&filter=active")
        .method(Method::GET)
        .to_srv_request();
    
    let auth_params = AuthUrlParams {
        csrf_token: CsrfToken::new("test_token".to_string()),
        nonce: Nonce::new("test_nonce".to_string()),
    };
    
    let state = OidcLoginState::new(&req, auth_params);
    assert_eq!(state.initial_url, "/dashboard.sql?user_id=123&filter=active");
}
```

## Usage Example

After this fix, the following flow now works correctly:

1. User visits: `https://example.com/report.sql?date=2024-01-01&format=pdf`
2. User is not authenticated, gets redirected to OIDC provider
3. User authenticates successfully 
4. User is redirected back to: `https://example.com/report.sql?date=2024-01-01&format=pdf` ✅

Previously, step 4 would redirect to: `https://example.com/report.sql` (losing query parameters) ❌

## Verification

To verify the fix works:

1. Set up OIDC authentication in SQLPage
2. Clear browser cookies to force re-authentication
3. Visit a SQLPage URL with query parameters (e.g., `/page.sql?param=value`)
4. Complete the OIDC authentication flow
5. Verify you're redirected back to the original URL with parameters intact

The fix preserves the user's original intent while maintaining security against open redirect attacks.