---
icon: hand-click
since: 0.14.0
category: forms-and-feedback
summary: Follow a link, download a file, or submit a form.
top:
  - class
  - name: justify
    type: TEXT
    description: The horizontal alignment of the button list (e.g., start, end, center, between).
  - name: shape
    type: TEXT
    description: Shape of the buttons (e.g., pill, square)
  - name: size
    type: TEXT
    description: The size of the buttons (e.g., sm, lg).
row:
  - name: color
    type: COLOR
    description: The color of the button (e.g., red, green, blue, but also primary, warning, danger, etc.).
      Only base color names are supported, not variations like "blue-lt" or "gray-300". Use a custom CSS stylesheet
      to further customize the colors.
  - name: disabled
    type: BOOLEAN
    description: Whether the button is disabled or not.
  - name: download
    type: TEXT
    description: If defined, the link will download the target instead of navigating to it. Set the value to
      the desired name of the downloaded file.
  - name: form
    type: TEXT
    description: Identifier (id) of the form to which the button should submit.
  - name: icon
    type: ICON
    description: Name of an icon to be displayed on the left side of the button.
  - name: icon_after
    type: ICON
    description: Name of an icon to display after the text in the button
  - name: id
    type: TEXT
    description: HTML Identifier to add to the button element.
  - name: image
    type: TEXT
    description: Path to image file (relative. relative to web root or URL) to be displayed on the button.
  - name: link
    type: URL
    description: The URL to which the button should navigate when clicked. If the form attribute is specified,
      then this overrides the page to which the form is submitted.
  - name: modal
    type: TEXT
    description: Display the modal window corresponding to the specified ID.
  - name: narrow
    type: BOOLEAN
    description: Whether to trim horizontal padding.
  - name: outline
    type: COLOR
    description: Outline color of the button (e.g. red, purple, ...)
  - name: rel
    type: TEXT
    description: '"nofollow" when the contents of the target link are not endorsed, "noopener" when the target
      is not trusted, and "noreferrer" to hide where the user came from when they open the link.'
  - name: space_after
    type: BOOLEAN
    description: Whether there should be extra space to the right of the button. In a line of buttons, this
      will put the buttons before this one on the left, and the ones after on the right.
  - name: target
    type: TEXT
    description: '"_blank" to open the link in a new tab, "_self" to open it in the same tab, "_parent" to
      open it in the parent frame, or "_top" to open it in the full body of the window.'
  - name: title
    type: TEXT
    description: The text displayed on the button.
  - name: tooltip
    type: TEXT
    description: Text displayed when the user hovers over the button.
---

Displays one or more buttons that follow links, download files, or submit a form.
