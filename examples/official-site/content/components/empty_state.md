---
icon: info-circle
since: 0.35.0
category: forms-and-feedback
summary: Explain an empty view and offer the reader a next step.
top:
  - class
  - name: description
    type: TEXT
    description: A short text displayed below the title.
  - name: description_md
    type: TEXT
    description: A short text displayed below the title, formatted using markdown.
  - name: header
    type: TEXT
    description: Text displayed on the top of the empty state.
  - name: icon
    type: ICON
    description: Name of an icon to be displayed on the top of the empty state.
  - id
  - name: image
    type: URL
    description: The URL (absolute or relative) of an image to display at the top of the empty state.
  - name: link
    type: URL
    required: true
    description: The URL to which the button should navigate when clicked.
  - name: link_icon
    type: ICON
    required: true
    description: Name of an icon to be displayed on the left side of the button.
  - name: link_text
    type: TEXT
    required: true
    description: The text displayed on the button.
  - name: title
    type: TEXT
    required: true
    description: Description of the empty state.
---

Displays a large placeholder message to communicate a single information to the user and invite them to take action.

Typically includes a title, an optional icon/image, descriptive text (rich text formatting and images supported via Markdown), and a call-to-action button.

Ideal for first-use screens, empty data sets, "no results" pages, or error messages.
