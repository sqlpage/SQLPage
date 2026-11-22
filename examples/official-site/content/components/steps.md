---
icon: dots-circle-horizontal
category: navigation
summary: Mark progress through the stages of a task.
top:
  - name: color
    type: COLOR
    description: Color of the bars displayed between steps.
  - name: counter
    type: TEXT
    description: Display the number of the step on top of its name.
  - name: description
    type: TEXT
    description: Description of the section.
  - name: id
    type: TEXT
    description: id attribute injected as an anchor in HTML. It can be used for scrolling to this item through
      links (use "#id" in link url). Added in v0.18.0.
  - name: title
    type: TEXT
    description: Title of the section.
row:
  - name: active
    type: BOOLEAN
    description: Whether this item in the grid is considered "active". Active items are displayed more prominently.
  - name: description
    type: TEXT
    description: Tooltip to display when the user passes their mouse over the step's name.
  - name: icon
    type: ICON
    description: An icon name (from tabler-icons.io) to display on the left side of the step name.
  - name: link
    type: URL
    description: A target URL to which the user should be taken when they click on the step.
  - name: title
    type: TEXT
    description: Name of the step.
---

Guide users through multi-stage processes, displaying a clear list of previous and future steps.
