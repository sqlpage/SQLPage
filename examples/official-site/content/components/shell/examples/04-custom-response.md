---
rows:
  - component: http_header
    Content-Type: application/xml
  - component: shell-empty
    contents: |-
      <?xml version="1.0"?>
       <user>
         <account>42</account>
         <login>john.doe</login>
       </user>
---

### Returning custom HTML, XML, plain text, or other formats

Use `shell-empty` to opt out of SQLPage's component system and return raw data directly.

By default, SQLPage wraps all your content in a complete HTML page with navigation and styling.
The `shell-empty` component tells SQLPage to skip this HTML wrapper and return only the raw content you specify.

Use it to create endpoints that return things like
 - XML (for JSON, use the [json](?component=json) component)
 - plain text or markdown content (for instance for consumption by LLMs)
 - a custom data format you need

When using `shell-empty`, you should use the [http_header](component.sql?component=http%5Fheader) component first
to set the correct [content type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type) (like `application/json` or `application/xml`).
