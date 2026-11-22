---
rows:
  - component: html
  - html: '<p>The following will be sanitized: <mark>'
    text: <script>alert('Failed XSS attack!')</script>
    post_html: </mark>. Phew! That was close!</p>
---

In order to include user-generated content that should be sanitized, use the `text` property instead of `html`. The `text` property will display the text as-is, without interpreting any HTML tags.
