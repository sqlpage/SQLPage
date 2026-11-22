---
icon: separator
since: 0.18.0
category: page-layout
summary: Separate sections with a horizontal rule.
top:
  - name: bold
    type: BOOLEAN
    description: Whether the text is bold.
  - class
  - name: color
    type: COLOR
    description: The name of a color for this span of text.
  - name: contents
    type: TEXT
    description: A text in the divider.
  - name: italics
    type: BOOLEAN
    description: Whether the text is italicized.
  - name: link
    type: URL
    description: URL of the link for the divider text. Available only when contents is present.
  - name: position
    type: TEXT
    description: Position of the text (e.g. left, right).
  - name: size
    type: INTEGER
    description: The size of the divider text, from 1 to 6.
  - name: underline
    type: BOOLEAN
    description: Whether the text is underlined.
---

Dividers help organize content and make the interface layout clear and uncluttered.
