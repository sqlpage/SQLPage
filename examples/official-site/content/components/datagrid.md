---
icon: grid-dots
category: display-data
summary: Lay out labeled values in a compact grid.
top:
  - class
  - name: description
    type: TEXT
    description: A short paragraph displayed below the title.
  - name: description_md
    type: TEXT
    description: A short paragraph displayed below the title - formatted using markdown.
  - name: icon
    type: ICON
    description: Name of an icon to display on the left side of the title.
  - id
  - name: image_url
    type: URL
    description: URL of an image to display on the left side of the title.
  - name: title
    type: TEXT
    description: Text header at the top of the data grid.
row:
  - name: active
    type: BOOLEAN
    description: Whether this item in the grid is considered "active". Active items are displayed more prominently.
  - name: color
    type: COLOR
    description: If set to a color name, the value will be displayed in a pill of that color.
  - name: description
    type: TEXT
    description: Value to display below the name.
  - name: footer
    type: TEXT
    description: Muted text to display below the value.
  - name: icon
    type: ICON
    description: An icon name (from tabler-icons.io) to display on the left side of the value.
  - id
  - name: image_url
    type: URL
    description: URL of a small image (such as an avatar) to display on the left side of the value.
  - name: link
    type: URL
    description: A target URL to which the user should be taken when they click on the value.
  - name: title
    type: TEXT
    required: true
    description: Name of the piece of information.
  - name: tooltip
    type: TEXT
    description: A tooltip to display when the user passes their mouse over the value.
---

Display small pieces of information in a clear and readable way. Each item has a name and is associated with a value.
