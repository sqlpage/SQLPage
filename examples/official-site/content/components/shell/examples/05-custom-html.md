---
rows:
  - component: shell-empty
    html: |-
      <!DOCTYPE html>
      <html>
      <head>
        <title>My page</title>
      </head>
      <body>
        <h1>My page</h1>
      </body>
      </html>
---

### Generate your own HTML
If you generate your own HTML from a SQL query, you can also use the `shell-empty` component to include it in a page.
This is useful when you want to generate a snippet of HTML that can be dynamically included in a larger page.
Make sure you know what you are doing, and be careful to escape the HTML properly,
as you are stepping out of the safe SQLPage framework and into the wild world of HTML.

In this scenario, you can use the `html` property, which serves as an alias for the `contents` property.
This property improves code readability by clearly indicating that you are generating HTML.
Since SQLPage returns HTML by default, there is no need to specify the content type in the HTTP header.
