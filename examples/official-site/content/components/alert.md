---
icon: alert-triangle
category: forms-and-feedback
summary: Draw attention to a message with a color and an icon.
top:
  - class
  - name: color
    type: COLOR
    description: The color theme for the alert message.
  - name: description
    type: TEXT
    description: Detailed description or content of the alert message.
  - name: description_md
    type: TEXT
    description: Detailed description or content of the alert message, in Markdown format, allowing you to
      use rich text formatting, including `**bold**` and `*italic*` text.
  - name: dismissible
    type: BOOLEAN
    description: Whether the user can close the alert message.
  - name: icon
    type: TEXT
    description: Icon name (from tabler-icons.io) to display next to the alert message.
  - id
  - name: important
    type: BOOLEAN
    description: Set this to TRUE to make the alert message more prominent.
  - name: link
    type: URL
    description: A URL to link to from the alert message.
  - name: link_text
    type: TEXT
    description: Customize the text of the link in the alert message. The default is "Ok".
  - name: title
    type: TEXT
    required: true
    description: Title of the alert message.
row:
  - name: color
    type: COLOR
    description: Customize the color of the link.
  - name: link
    type: URL
    description: A URL to link to from the alert message.
  - name: title
    type: TEXT
    description: Customize the text of the link in the alert message. The default is "Ok".
---

A visually distinctive message or notification.
