---
icon: dots
since: 0.18.0
category: navigation
summary: Link the current page to its parent pages.
top:
  - class
  - id
row:
  - name: active
    type: TEXT
    description: Whether the link is active or not. Defaults to false.
  - name: description
    type: TEXT
    description: Description of the link. This is displayed when the user hovers over the link.
  - name: link
    type: TEXT
    description: Link to the page to display when the link is clicked. By default, the link refers to the current
      page, with a 'link' parameter set to the link's title.
  - name: title
    type: TEXT
    required: true
    description: Hyperlink text to display.
---

A secondary navigation aid that helps users understand their location on a website or mobile application.
