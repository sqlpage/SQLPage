---
icon: html
since: 0.25.0
category: page-layout
summary: Insert your own HTML markup into the page.
top:
  - name: html
    type: TEXT
    description: Raw HTML content to include in the page. This will not be sanitized or escaped. If you include
      content from an untrusted source, your page will be vulnerable to cross-site scripting attacks.
row:
  - name: html
    type: TEXT
    description: Raw HTML content to include in the page. This will not be sanitized or escaped. If you include
      content from an untrusted source, your page will be vulnerable to cross-site scripting attacks.
  - name: post_html
    type: TEXT
    description: Raw HTML content to include after the text content. This will not be sanitized or escaped.
      If you include content from an untrusted source, your page will be vulnerable to cross-site scripting
      attacks.
  - name: text
    type: TEXT
    description: Text content to include in the page. This will be sanitized and escaped. Use this property
      to include user-generated content that should not contain HTML tags.
---

Include raw HTML in the output. For advanced users only. Use this component to create complex layouts or to include external content.
    Be very careful when using this component with user-generated content, as it can lead to security vulnerabilities.
    Use this component only if you are familiar with the security implications of including raw HTML, and understand the risks of cross-site scripting (XSS) attacks.
