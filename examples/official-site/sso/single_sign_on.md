# Setting Up Single Sign-On in SQLPage

When you want to add user authentication to your SQLPage application, you have two main options:

1. The [authentication component](/component.sql?component=authentication):
   A simple username/password system, that you have to manage yourself.
2. **OpenID Connect (OIDC)**:
   A single sign-on system that lets users log in with their existing accounts (like Google, Microsoft, or your organization's own identity provider).

This guide will help you set up single sign-on using OpenID connect with SQLPage quickly.

## Essential Terms

- **OIDC** ([OpenID Connect](https://openid.net/developers/how-connect-works/)): The protocol that enables secure login with existing accounts. While it adds some complexity, it's an industry standard that ensures your users' data stays safe.
- **Issuer** (or identity provider): The service that verifies your users' identity (like Google or Microsoft)
- **Identity Token**: A secure message from the issuer containing user information. It is stored as a cookie on the user's computer, and sent with every request after login. SQLPage will redirect all requests that do not contain a valid token to the identity provider's login page.
- **Claim**: A piece of information contained in the token about the user (like their name or email)

## Quick Setup Guide

### Choose an OIDC Provider

Here are the setup guides for
[Google](https://developers.google.com/identity/openid-connect/openid-connect),
[Microsoft Entra ID](https://learn.microsoft.com/en-us/entra/identity-platform/quickstart-register-app),
and [Keycloak](https://www.keycloak.org/getting-started/getting-started-docker) (self-hosted).

### Register Your Application

1. Go to your chosen provider's developer console
2. Create a new application
3. Set the redirect URI to `http://localhost:8080/sqlpage/oidc_callback`. (We will change that later when you deploy your site to a hosting provider such as [datapage](https://beta.datapage.app/)).
4. Note down the client ID and client secret

### Configure SQLPage

Create or edit `sqlpage/sqlpage.json` to add the following configuration keys:

```json
{
  "oidc_issuer_url": "https://accounts.google.com",
  "oidc_client_id": "your-client-id",
  "oidc_client_secret": "your-client-secret",
  "host": "localhost:8080"
}
```

#### Provider-specific settings
- Google: `https://accounts.google.com`
- Microsoft: `https://login.microsoftonline.com/{tenant}/v2.0`. [Find your value of `{tenant}`](https://learn.microsoft.com/en-us/entra/identity-platform/quickstart-create-new-tenant).
- GitHub: `https://github.com`
- Keycloak: Use [your realm's base url](https://www.keycloak.org/securing-apps/oidc-layers), ending in `/auth/realms/{realm}`.
- For other OIDC providers, you can usually find the issuer URL by
  looking for a "discovery document" or "well-known configuration" at an URL that ends with the suffix `/.well-known/openid-configuration`.
  Strip the suffix and use it as the `oidc_issuer_url`.

### Restart SQLPage

When you restart your SQLPage instance, it should automatically contact
the identity provider, find its login URL, and the public keys that will be used to check the validity of its identity tokens.

By default, all pages on your website will now require users to log in.

## Access User Information in Your SQL

Once you have successfully configured SSO, you can access information
about the authenticated user who is visiting the current page using the following functions:
- [`sqlpage.user_info`](/functions.sql?function=user_info) to access a particular claim about the user such as `name` or `email`,
- [`sqlpage.user_info_token`](/functions.sql?function=user_info_token) to access the entire identity token as json.

Access user data in your SQL files:

```sql
select 'text' as component, '

Welcome, ' || sqlpage.user_info('name') || '!

You have visited this site ' || 
    (select count(*) from page_visits where user=sqlpage.user_info('sub')) ||
' times before.
' as contents_md;

insert into page_visits
  (path, user)
values
  (sqlpage.path(), sqlpage.user_info('sub'));
```

## Restricting authentication to a specific set of pages

Sometimes, you don't want to protect your entire website with a login, but only a specific section.
You can achieve this by adding the `oidc_protected_paths` option to your `sqlpage.json` file.

This option takes a list of URL prefixes. If a user requests a page whose address starts with one of these prefixes, they will be required to log in.

**Example:** Protect only pages in the `/admin` folder.

```json
{
  "oidc_issuer_url": "https://accounts.google.com",
  "oidc_client_id": "your-client-id",
  "oidc_client_secret": "your-client-secret",
  "host": "localhost:8080",
  "oidc_protected_paths": ["/admin"]
}
```

In this example, a user visiting `/admin/dashboard.sql` will be prompted to log in, while a user visiting `/index.sql` will not.

## Going to Production

When deploying to production:

1. Update the redirect URI in your OIDC provider's settings to:
   ```
   https://your-domain.com/sqlpage/oidc_callback
   ```

2. Update your `sqlpage.json`:
   ```json
   {
     "oidc_issuer_url": "https://accounts.google.com",
     "oidc_client_id": "your-client-id",
     "oidc_client_secret": "your-client-secret",
     "host": "your-domain.com"
   }
   ```

3. If you're using HTTPS (recommended), make sure your `host` setting matches your domain name exactly.

## Troubleshooting

### Version Requirements
- OIDC support requires SQLPage **version 0.35 or higher**. Check your version in the logs.

### Common Configuration Issues
- **Redirect URI Mismatch**: The redirect URI in your OIDC provider settings must exactly match `https://your-domain.com/sqlpage/oidc_callback` (or `http://localhost:8080/sqlpage/oidc_callback` for local development)
- **Invalid Client Credentials**: Double-check your client ID and secret are copied correctly from your OIDC provider
- **Host Configuration**: The `host` setting in `sqlpage.json` must match your application's domain name exactly
- **HTTPS Requirements**: Most OIDC providers require HTTPS in production. Ensure your site is served over HTTPS.
- **Provider Discovery**: If SQLPage fails to discover your provider's configuration, verify the `oidc_issuer_url` is correct and accessible by loading `{oidc_issuer_url}/.well-known/openid-configuration` in your browser.

### Debugging Tips
- Check SQLPage's logs for detailed error messages. You can enable verbose logging with the `RUST_LOG=trace` environment variable.
- Verify your OIDC provider's logs for authentication attempts
- In production, confirm your domain name matches exactly in both the OIDC provider settings and `sqlpage.json`
- If [using a reverse proxy](/your-first-sql-website/nginx.sql), ensure it's properly configured to handle the OIDC callback path.
- If you have checked everything and you think the bug comes from SQLPage itself, [open an issue on our bug tracker](https://github.com/sqlpage/SQLPage/issues).
