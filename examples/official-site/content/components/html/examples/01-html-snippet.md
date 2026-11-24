---
rows:
  - component: html
    html: <h1 class="text-blue">This text is safe because it is <mark>hardcoded</mark>!</h1>
---

Include a simple HTML snippet. In this example, the HTML code is hardcoded in the SQL query, so it is safe. You should never include data that may be manipulated by a user in the HTML content.
