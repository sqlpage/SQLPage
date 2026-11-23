---
icon: letter-case-upper
since: 0.19.0
category: page-layout
summary: Add a heading and choose its level.
top:
  - name: center
    type: BOOLEAN
    description: Whether to center the title.
  - class
  - name: contents
    type: TEXT
    required: true
    description: A text to display.
  - id
  - name: level
    type: INTEGER
    description: Set the heading level (default level is 1)
---

Defines HTML headings. The level 1 is used for the maximal size and the level 6 is used for the minimal size.
