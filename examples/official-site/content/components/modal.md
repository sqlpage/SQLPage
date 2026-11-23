---
icon: app-window
since: 0.36.0
category: forms-and-feedback
summary: Open a dialog over the page for details, forms, or media.
top:
  - name: allow
    type: TEXT
    description: For embedded content, this attribute specifies the features or permissions that can be used.
  - name: class
    type: TEXT
    description: Class attribute added to the container in HTML. It can be used to apply custom styling to
      this item through css.
  - name: close
    type: TEXT
    description: The text to display in the Close button.
  - name: close_label
    type: TEXT
    description: The same as `close`. Use it on databases where `close` is a reserved word, such as SQL Server.
  - name: embed
    type: TEXT
    description: Embed remote content in an iframe.
  - name: embed_mode
    type: TEXT
    description: Use "iframe" to display embedded content within an iframe.
  - name: height
    type: INTEGER
    description: Height of the modal content, in pixels. Overflow content scrolls.
  - name: id
    type: TEXT
    required: true
    description: ID attribute added to the container in HTML. It can be used to target this item through css
      or for displaying this item.
  - name: large
    type: BOOLEAN
    description: Indicates that the modal box has an increased width.
  - name: sandbox
    type: TEXT
    description: For embedded content, this attribute specifies the security restrictions on the loaded content.
  - name: scrollable
    type: BOOLEAN
    description: Create a scrollable modal that allows scroll the modal body.
  - name: small
    type: BOOLEAN
    description: Indicates that the modal box has a reduced width.
  - name: style
    type: TEXT
    description: Applies CSS styles to the embedded content.
  - name: title
    type: TEXT
    required: true
    description: Description of the modal box.
row:
  - name: contents
    type: TEXT
    description: A paragraph of text to display, without any formatting, without having to make additional
      queries.
  - name: contents_md
    type: TEXT
    description: Rich text in the markdown format. Among others, this allows you to write bold text using `**bold**`,
      italics using `*italics*`, and links using `[text](https://example.com)`.
---

Defines the a temporary popup box displayed on top of a webpage’s content.
Useful for displaying additional information, help, or collect data from users.

Modals are closed by default, and can be opened by clicking on a button or link targeting their ID.
