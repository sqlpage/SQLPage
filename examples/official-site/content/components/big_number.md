---
icon: chart-area
since: 0.28.0
category: display-data
summary: Highlight a metric with a trend or progress indicator.
top:
  - class
  - name: columns
    type: INTEGER
    description: The number of columns to display the big numbers in (default is one column per item).
  - id
  - name: wrap
    type: BOOLEAN
    description: Wrap long titles and descriptions onto multiple lines. By default, they are truncated with
      an ellipsis.
row:
  - name: change_percent
    type: INTEGER
    description: The percentage change in value (e.g., 7 for 7% increase, -8 for 8% decrease).
  - class
  - name: color
    type: COLOR
    description: The color of the card background and its main value.
  - name: description
    type: TEXT
    description: A description or additional context for the big number.
  - name: dropdown_item
    type: JSON
    description: A list of JSON objects containing links. e.g. {"label":"This week", "link":"?days=7"}
  - id
  - name: progress_color
    type: TEXT
    description: The color of the progress bar (e.g., "primary", "success", "danger").
  - name: progress_percent
    type: INTEGER
    description: The value of the progress (0-100).
  - name: title
    type: TEXT
    description: The title or label for the big number.
  - name: title_link
    type: TEXT
    description: A link for the Big Number title. If set, the entire title becomes clickable.
  - name: title_link_new_tab
    type: BOOLEAN
    description: If true, the title link will open in a new tab/window.
  - name: unit
    type: TEXT
    description: The unit of measurement for the value.
  - name: value
    type: TEXT
    required: true
    description: The main value to be displayed prominently.
  - name: value_link
    type: TEXT
    description: A link for the Big Number value. If set, the entire value becomes clickable.
  - name: value_link_new_tab
    type: BOOLEAN
    description: If true, the value link will open in a new tab/window.
---

A component to display key metrics or statistics with optional description, change indicator, and progress bar. Useful in dashboards.
