---
icon: logout
since: 0.41.0
category: identity-and-security
summary: Build a sign-out URL for an OpenID Connect session.
parameters:
  - name: redirect_uri
    type: TEXT
    description: The relative URL path where the user should be redirected after logout. Must start with `/`.
      Defaults to `/` if not provided.
---

The `sqlpage.oidc_logout_url` function generates a secure logout URL for users authenticated via [OIDC Single Sign-On](/sso).

When a user visits this URL, SQLPage will:
1. Remove the authentication cookie
2. Redirect the user to the OIDC provider's logout endpoint (if available)
3. Finally redirect back to the specified `redirect_uri`

## Security Features

This function provides protection against **Cross-Site Request Forgery (CSRF)** attacks:
- The generated URL contains a cryptographically signed token
- The token includes a timestamp and expires after 10 minutes
- The token is signed using your OIDC client secret
- Only relative URLs (starting with `/`) are allowed as redirect targets

This means that malicious websites cannot trick your users into logging out by simply including an image or link to your logout URL.

## Add a logout button

```sql
select 'button' as component;
select
    'Logout' as title,
    sqlpage.oidc_logout_url('/') as link,
    'logout' as icon,
    'red' as outline;
```

This creates a logout button that, when clicked:
1. Logs the user out of your SQLPage application
2. Logs the user out of the OIDC provider (if the provider supports [RP-Initiated Logout](https://openid.net/specs/openid-connect-rpinitiated-1_0.html))
3. Redirects the user back to your homepage (`/`)

## Examples

### Logout Button in Navigation

```sql
select 'shell' as component,
    'My App' as title,
    json_array(
        json_object(
            'title', 'Logout',
            'link', sqlpage.oidc_logout_url('/'),
            'icon', 'logout'
        )
    ) as menu_item;
```

### Logout with Return to Current Page

```sql
select 'button' as component;
select
    'Sign Out' as title,
    sqlpage.oidc_logout_url(sqlpage.path()) as link;
```

### Conditional Logout Link

```sql
select 'button' as component
where sqlpage.user_info('sub') is not null;
select
    'Logout ' || sqlpage.user_info('name') as title,
    sqlpage.oidc_logout_url('/') as link
where sqlpage.user_info('sub') is not null;
```

## Requirements

- OIDC must be [configured](/sso) in your `sqlpage.json`
- If OIDC is not configured, this function returns NULL
- The `redirect_uri` must be a relative path starting with `/`

## Provider Support

The logout behavior depends on your OIDC provider:

| Provider | Full Logout Support |
|----------|-------------------|
| Keycloak | ✅ Yes |
| Auth0 | ✅ Yes |
| Google | ❌ No (local logout only) |
| Azure AD | ✅ Yes |
| Okta | ✅ Yes |

When the provider doesn't support RP-Initiated Logout, SQLPage will still remove the local authentication cookie and redirect to your specified URI.
