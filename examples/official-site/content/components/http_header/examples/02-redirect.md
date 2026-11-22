---
rows:
  - component: http_header
    Location: /another-page.sql
---

Redirect the user to another page. In this example, the user is redirected to a file named another-page.sql at the root of the website. The current page will not be displayed at all.
        This is useful in particular for content creation pages that contain only INSERT statements, because you can redirect the user to the page that lists the content after it has been created.
