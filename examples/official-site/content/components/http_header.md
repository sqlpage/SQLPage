---
icon: world-www
category: http-responses
summary: Set response headers for caching, content types, and more.
top:
  - name: Access-Control-Allow-Origin
    type: TEXT
    description: Specifies which origins are allowed to access the resource in a cross-origin request, used
      for implementing Cross-Origin Resource Sharing (CORS).
  - name: Cache-Control
    type: TEXT
    description: Directives for how long the page should be cached by the browser. Set this to max-age=N to
      keep the page in cache for N seconds.
  - name: Content-Disposition
    type: TEXT
    description: Provides instructions on how the response content should be displayed or handled by the client,
      such as inline or as an attachment.
  - name: Location
    type: TEXT
    description: Specifies the URL to redirect the client to, usually used in 3xx redirection responses.
  - name: Set-Cookie
    type: TEXT
    description: Sets a cookie in the client browser, used for session management and storing user-related
      information.
---

An advanced component to set arbitrary HTTP headers: can be used to set a custom caching policy to your pages, or implement custom redirections, for example.
If you are a beginner, you probably don't need this component.

When used, this component has to be the first component in the page, because once the page is sent to the browser, it is too late to change the headers.

HTTP headers are additional pieces of information sent with responses to web requests that provide instructions
or metadata about the data being sent — for example,
setting cache control directives to control caching behavior
or specifying the content type of a response.

Any valid HTTP header name can be used as a top-level parameter for this component.
The examples shown here are just that, examples; and you can create any custom header
if needed simply by declaring it.

If your header's name contains a dash or any other special character,
you will have to use your database's quoting mechanism to declare it.
In standard SQL, you can use double quotes to quote identifiers (like "X-My-Header"),
in Microsoft SQL Server, you can use square brackets (like [X-My-Header]).
