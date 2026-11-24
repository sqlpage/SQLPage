---
icon: timeline-event-text
since: 0.18.0
category: display-data
summary: Show a sequence of activity or status changes as colored segments.
top:
  - name: center
    type: BOOLEAN
    description: Whether to center the component.
  - class
  - name: description
    type: TEXT
    description: A short paragraph.
  - name: description_md
    type: TEXT
    description: A short paragraph formatted using markdown.
  - id
  - name: information
    type: TEXT
    description: A short text displayed below the title.
  - name: placement
    type: TEXT
    description: Position of the tooltip (e.g. top, bottom, right, left)
  - name: title
    type: TEXT
    required: true
    description: Title of the tracking component.
  - name: width
    type: INTEGER
    description: Width of the component, between 1 and 12.
row:
  - name: color
    type: TEXT
    description: Color of the tracked item (e.g. success, warning, danger)
  - name: link
    type: URL
    description: A URL to open when the user clicks on the tracked item.
  - name: title
    type: TEXT
    required: true
    description: Description of the state.
---

Component for visualising activity logs or other monitoring-related data.
