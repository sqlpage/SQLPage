---
icon: lock
since: 0.7.2
category: server-logic
summary: Restrict access by checking a password against a stored hash.
top:
  - name: link
    type: TEXT
    description: The URL to redirect the user to if they are not logged in. If this parameter is not specified,
      the user will stay on the current page, but be asked to log in using a popup in their browser (HTTP basic
      authentication).
  - name: password
    type: TEXT
    description: The password that was sent by the user. You can set this to :password if you have a login
      form leading to your page.
  - name: password_hash
    type: TEXT
    description: The hash of the password that you stored for the user that is currently trying to log in.
      These hashes can be generated ahead of time using a tool like https://argon2.online/.
---

Create pages with password-restricted access.


When you want to add user authentication to your SQLPage application,
you have two main options:

1. The `authentication` component:
   - lets you manage usernames and passwords yourself
   - does not require any external service
   - gives you fine-grained control over
     - which pages and actions are protected
     - the look of the [login form](?component=login)
     - the duration of the session
     - the permissions of each user
2. [**Single sign-on**](/sso)
   - lets users log in with their existing accounts (like Google, Microsoft, or your organization's own identity provider)
   - requires setting up an external service (Google, Microsoft, etc.)
   - frees you from implementing a lot of features like password reset, account creation, user management, etc.

This page describes the first option.

When used, this component has to be at the top of your page,
because once the page has begun being sent to the browser,
it is too late to restrict access to it.

The authentication component checks if the user has sent the correct password,
and if not, redirects them to the URL specified in the link parameter.

If you don't want to re-check the password on every page (which is an expensive operation),
you can check the password only once and store a session token in your database
(see the session example below).

You can use the [cookie component](?component=cookie) to set the session token cookie in the client browser,
and then check whether the token matches what you stored in subsequent pages.
