---
icon: columns
since: 0.29.0
category: page-layout
summary: Compare plans, features, or options in side-by-side cards.
row:
  - name: button_color
    type: TEXT
    description: Optional color for the button.
  - name: button_text
    type: TEXT
    description: Text for the button.
  - name: description
    type: TEXT
    description: A brief description of the item.
  - name: description_md
    type: TEXT
    description: A brief description of the item, formatted using markdown.
  - name: icon
    type: ICON
    description: Optional icon to display in a ribbon.
  - name: icon_color
    type: TEXT
    description: Color for the icon in the ribbon.
  - name: item
    type: JSON
    description: A list of bullet points associated with the columns, represented either as text, or as a json
      object with "icon", "color", and "description" or "description_md" fields.
  - name: link
    type: TEXT
    description: A link associated with the item.
  - name: size
    type: INTEGER
    description: Size of the column, affecting layout.
  - name: small_text
    type: TEXT
    description: Optional small text to display after the value.
  - name: target
    type: TEXT
    description: Optional target for the button. Set to "_blank" to open links in a new tab.
  - name: title
    type: TEXT
    description: The title or label for the item.
  - name: value
    type: TEXT
    description: The value associated with the item.
  - name: value_color
    type: TEXT
    description: Color for the value text.
---

A component to display various items in a card layout, allowing users to choose options. Useful for showcasing different features or services, or KPIs. See also the big_number component.
