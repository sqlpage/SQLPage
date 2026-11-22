---
icon: cookie
category: http-responses
summary: Set a cookie in the visitor's browser.
top:
  - name: domain
    type: TEXT
    description: The domain for which the cookie will be sent. If not specified, the cookie will be sent for
      all domains.
  - name: expires
    type: TIMESTAMP
    description: The date at which the cookie expires (either a timestamp or a date object). If not specified,
      the cookie will expire when the browser is closed.
  - name: http_only
    type: BOOLEAN
    description: Whether the cookie should only be accessible via HTTP and not via client-side scripts. If
      explicitly set to false, the cookie will be accessible via both HTTP and client-side scripts.
  - name: max_age
    type: INTEGER
    description: The maximum age of the cookie in seconds. number of seconds until the cookie expires. If both
      Expires and Max-Age are set, Max-Age has precedence.
  - name: name
    type: TEXT
    required: true
    description: The name of the cookie to set.
  - name: path
    type: TEXT
    description: The path for which the cookie will be sent. If not specified, the cookie will be sent for
      all paths.
  - name: remove
    type: BOOLEAN
    description: Set to TRUE to remove the cookie from the client browser. When specified, other parameters
      are ignored.
  - name: same_site
    type: TEXT
    description: Whether the cookie should only be sent for requests originating from the same site. See owasp.org/www-community/SameSite.
      `strict` is the recommended and default value, but you may want to set it to `lax` if you want your users
      to keep their session when they click on a link to your site from an external site.
  - name: secure
    type: BOOLEAN
    description: Whether the cookie should only be sent over a secure (HTTPS) connection. Defaults to TRUE.
  - name: value
    type: TEXT
    description: The value of the cookie to set.
---

Sets a cookie in the client browser, used for session management and storing user-related information.

This component creates a single cookie. Since cookies need to be set before the response body is sent to the client,
this component should be **placed at the top of the page**, before any other components that generate output.

After being set, a cookie can be accessed anywhere in your SQL code using the `sqlpage.cookie('cookie_name')` pseudo-function.

Note that if your site is accessed over HTTP (and not HTTPS), you have to set `false as secure` to force browsers to accept your cookies.
