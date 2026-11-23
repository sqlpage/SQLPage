---
rows:
  - component: http_header
    Cache-Control: public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400
---

Set cache control directives for caching behavior. In this example, the response can be cached by the browser
        and served from the cache for up to 600 seconds (10 minutes) after it is first requested.
        During that time, even if the cached response becomes stale (outdated), the browser can still use it (stale-while-revalidate)
        for up to 3600 seconds (1 hour) while it retrieves a fresh copy from the server in the background.
        If there is an error while trying to retrieve a fresh copy from the server,
        the browser can continue to serve the stale response for up to 86400 seconds (24 hours) (stale-if-error) instead of showing an error page.
        This caching behavior helps improve the performance and responsiveness of the website by reducing the number of requests made to the server
        and allowing the browser to serve content from its cache when appropriate.
