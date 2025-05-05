INSERT INTO
    sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES
    (
        'user_info_token',
        '0.35.0',
        'key',
        '# Accessing information about the current user, when logged in with SSO

This function can be used only when you have [configured Single Sign-On with an OIDC provider](/sso).

## The ID Token

When a user logs in through OIDC, your application receives an [identity token](https://openid.net/specs/openid-connect-core-1_0.html#IDToken) from the identity provider.
This token contains information about the user, such as their name and email address.
The `sqlpage.user_info_token()` function lets you access the entire contents of the ID token, as a JSON object.
You can then use [your database''s JSON functions](/blog.sql?post=JSON+in+SQL%3A+A+Comprehensive+Guide) to process that JSON.

If you need to access a specific claim, it is easier and more performant to use the
[`sqlpage.user_info()`](?function=user_info) function instead.

### Example: Displaying User Information

```sql
select ''list'' as component;
select key as title, value as description
from json_each(sqlpage.user_info_token());
```

This sqlite-specific example will show all the information available about the current user, such as:
- `sub`: A unique identifier for the user
- `name`: The user''s full name
- `email`: The user''s email address
- `picture`: A URL to the user''s profile picture

### Security Notes

- The ID token is automatically verified by SQLPage to ensure it hasn''t been tampered with.
- The token is only available to authenticated users: if no user is logged in or sso is not configured, this function returns NULL
- If some information is not available in the token, you have to configure it on your OIDC provider, SQLPage can''t do anything about it.
- The token is stored in a signed http-only cookie named `sqlpage_auth`. You can use [the cookie component](/component.sql?component=cookie) to delete it, and the user will be redirected to the login page on the next page load.
'
    );

INSERT INTO
    sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES
    (
        'user_info',
        '0.34.0',
        'user',
        '# Accessing Specific User Information

The `sqlpage.user_info` function is a convenient way to access specific pieces of information about the currently logged-in user.
When you [configure Single Sign-On](/sso), your OIDC provider will issue an [ID token](https://openid.net/specs/openid-connect-core-1_0.html#IDToken) for the user,
which contains *claims*, with information about the user.

Calling `sqlpage.user_info(claim_name)` lets you access these claims directly from SQL.

## How to Use

The function takes one parameter: the name of the *claim* (the piece of information you want to retrieve).

For example, to display a personalized welcome message, with the user''s name, you can use:

```sql
select ''text'' as component;
select ''Welcome, '' || sqlpage.user_info(''name'') || ''!'' as title;
```

## Available Information

The exact information available depends on your identity provider (the service you chose to authenticate with),
its configuration, and the scopes you requested.
Use [`sqlpage.user_info_token()`](?function=user_info_token) to see all the information available in the ID token of the current user.

Here are some commonly available fields:

### Basic Information
- `name`: The user''s full name (usually first and last name separated by a space)
- `email`: The user''s email address (*warning*: there is no guarantee that the user currently controls this email address. Use the `sub` claim for database references instead.)
- `picture`: URL to the user''s profile picture

### User Identifiers
- `sub`: A unique identifier for the user (use this to uniquely identify the user in your database)
- `preferred_username`: The username the user prefers to use

### Name Components
- `given_name`: The user''s first name
- `family_name`: The user''s last name

## Examples

### Personalized Welcome Message
```sql
select ''text'' as component,
    ''Welcome back, **'' || sqlpage.user_info(''given_name'') || ''**!'' as contents_md;
```

### User Profile Card
```sql
select ''card'' as component;
select 
    sqlpage.user_info(''name'') as title,
    sqlpage.user_info(''email'') as description,
    sqlpage.user_info(''picture'') as image;
```

### Conditional Content Based on custom claims

Some identity providers let you add custom claims to the ID token.
This lets you customize the behavior of your application based on arbitrary user attributes,
such as the user''s role.

```sql
-- show everything to admins, only public items to others
select ''list'' as component;
select title from my_items
  where is_public or sqlpage.user_info(''role'') = ''admin''
```

## Security Best Practices

> ⚠️ **Important**: Always use the `sub` claim to identify users in your database, not their email address.
> The `sub` claim is guaranteed to be unique and stable for each user, while email addresses can change.
> In most providers, receiving an id token with a given email does not guarantee that the user currently controls that email.

```sql
-- Store the user''s ID in your database
insert into user_preferences (user_id, theme)
values (sqlpage.user_info(''sub''), ''dark'');
```

## Troubleshooting

If you''re not getting the information you expect:

1. Check that OIDC is properly configured in your `sqlpage.json`
2. Verify that you requested the right scopes in your OIDC configuration
3. Try using `sqlpage.user_info_token()` to see all available information
4. Check your OIDC provider''s documentation for the exact claim names they use

Remember: If the user is not logged in or the requested information is not available, this function returns NULL.
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
        'user_info',
        1,
        'claim',
        'The name of the user information to retrieve. Common values include ''name'', ''email'', ''picture'', ''sub'', ''preferred_username'', ''given_name'', and ''family_name''. The exact values available depend on your OIDC provider and configuration.',
        'TEXT'
    );