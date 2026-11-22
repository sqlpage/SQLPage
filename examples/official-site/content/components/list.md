---
icon: list
category: display-data
summary: Present rows as a vertical list with optional links and icons.
top:
  - class
  - name: compact
    type: BOOLEAN
    description: Whether to display the list in a more compact format, allowing more items to be displayed
      on the screen.
  - name: empty_description
    type: TEXT
    description: Description to display if the list is empty.
  - name: empty_description_md
    type: TEXT
    description: Description to display if the list is empty, in Markdown format.
  - name: empty_link
    type: URL
    description: URL to which the user should be taken if they click on the empty list.
  - name: empty_title
    type: TEXT
    description: Title text to display if the list is empty.
  - id
  - name: title
    type: TEXT
    description: Text header at the top of the list.
  - name: wrap
    type: BOOLEAN
    description: Wrap list items onto multiple lines if they are too long
row:
  - name: active
    type: BOOLEAN
    description: Whether this item in the list is considered "active". Active items are displayed more prominently.
  - class
  - name: color
    type: COLOR
    description: The name of a color, to be displayed as a dot near the list item contents.
  - name: delete_link
    type: URL
    description: A page that will be loaded when the user clicks on the delete button for this specific item.
      The link will be submitted as a POST request.
  - name: description
    type: TEXT
    description: A description of the list item, displayed as greyed-out text.
  - name: description_md
    type: TEXT
    description: A description of the list item, displayed as greyed-out text, in Markdown format, allowing
      you to use rich text formatting, including `**bold**` and `*italic*` text.
  - name: edit_link
    type: URL
    description: A URL to which the user should be taken when they click on the "edit" icon. Does not show
      the icon when omitted.
  - name: icon
    type: ICON
    description: Name of an icon to display on the left side of the item.
  - id
  - name: image_url
    type: URL
    description: The URL of a small image to display on the left side of the item.
  - name: link
    type: URL
    description: An URL to which the user should be taken when they click on the list item.
  - name: title
    type: TEXT
    required: true
    description: Name of the list item, displayed prominently.
  - name: view_link
    type: URL
    description: A URL to which the user should be taken when they click on the "view" icon. Does not show
      the icon when omitted.
---

A vertical list of items. Each item can be clickable and link to another page.
