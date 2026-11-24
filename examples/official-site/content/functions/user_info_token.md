---
icon: key
since: 0.35.0
category: identity-and-security
summary: Read all claims from the user's OpenID Connect ID token.
---

This function can be used only when you have [configured Single Sign-On with an OIDC provider](/sso).

## The ID Token

When a user logs in through OIDC, your application receives an [identity token](https://openid.net/specs/openid-connect-core-1_0.html#IDToken) from the identity provider.
This token contains information about the user, such as their name and email address.
The `sqlpage.user_info_token()` function lets you access the entire contents of the ID token, as a JSON object.
You can then use [your database's JSON functions](/blog.sql?post=JSON+in+SQL%3A+A+Comprehensive+Guide) to process that JSON.

If you need to access a specific claim, it is easier and more performant to use the
[`sqlpage.user_info()`](?function=user_info) function instead.

## Displaying user claims

```sql
select 'list' as component;
select key as title, value as description
from json_each(sqlpage.user_info_token());
```

This sqlite-specific example will show all the information available about the current user, such as:
- `sub`: A unique identifier for the user
- `name`: The user's full name
- `email`: The user's email address
- `picture`: A URL to the user's profile picture

## Security notes

- The ID token is automatically verified by SQLPage to ensure it hasn't been tampered with.
- The token is only available to authenticated users: if no user is logged in or sso is not configured, this function returns NULL
- If some information is not available in the token, you have to configure it on your OIDC provider, SQLPage can't do anything about it.
- The token is stored in a signed http-only cookie named `sqlpage_auth`. You can use [the cookie component](/component.sql?component=cookie) to delete it, and the user will be redirected to the login page on the next page load.
