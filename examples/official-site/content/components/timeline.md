---
icon: git-commit
since: 0.13.0
category: display-data
summary: Arrange events along a vertical timeline.
top:
  - class
  - id
  - name: simple
    type: BOOLEAN
    description: If set to true, the timeline will be displayed in a condensed format without icons.
row:
  - class
  - name: color
    type: TEXT
    description: Color of the icon. See preview.tabler.io/colors.html for a list of available colors.
  - name: date
    type: TEXT
    required: true
    description: Date of the event.
  - name: description
    type: TEXT
    description: Textual description of the event.
  - name: description_md
    type: TEXT
    description: Description of the event in Markdown.
  - name: icon
    type: TEXT
    description: Name of the icon to display next to the event. See tabler-icons.io for a list of available
      icons.
  - id
  - name: link
    type: TEXT
    description: Link to a page with more information about the event.
  - name: title
    type: TEXT
    required: true
    description: Name of the event.
---

A list of events with a vertical line connecting them.
