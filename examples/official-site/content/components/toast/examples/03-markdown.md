---
rows:
  - component: toast
    id: toast-markdown
    trigger: rich-notifications
    title: Release available
    description: <strong>This fallback stays escaped</strong>
    description_md: Version **2.0** is ready. [Read the notes](https://example.com/releases).
    color: blue
    duration: 0
  - component: toast
    id: toast-plain
    trigger: rich-notifications
    description: <strong>Plain text stays escaped</strong>
    color: white
    duration: 0
  - component: button
  - title: Show rich notifications
    link: '#rich-notifications'
---

A persistent rich Markdown notification. Markdown takes precedence over plain text and renders emphasis and a link as HTML, while plain-text content remains escaped.
