# Single Sign-On with OpenID Connect

Welcome to this demonstration of how to implement *OpenID Connect* (OIDC) authentication in a SQLPage application.

[OIDC](https://openid.net/connect/) is a standard authentication protocol that allows users to authenticate with a third-party identity provider and then access applications without having to log in again. This is useful for single sign-on (SSO) scenarios where users need to access multiple applications with a single set of credentials. OIDC can be used to implement a "Login with Google" or "Login with Facebook" button in your application, since these providers support the OIDC protocol.

To test this application, click the login button on the top right corner of the page.
You will be redirected to the identity provider's login page, where you can login with the following credentials:
- **Username: `demo`**
- **Password: `demo`**

After logging in, you will be redirected back to this page, and you will see the user information that was returned by the identity provider.

![closed](/assets/closed.jpeg)
