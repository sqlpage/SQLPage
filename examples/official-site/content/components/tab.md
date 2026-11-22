---
icon: row-insert-bottom
since: 0.9.5
category: navigation
summary: Switch between related pages using labeled tabs.
top:
  - name: center
    type: BOOLEAN
    description: Whether the tabs should be centered or not. Defaults to false.
  - class
  - id
row:
  - name: active
    type: BOOLEAN
    description: Whether the tab is active or not. Defaults to false.
  - name: center
    type: BOOLEAN
    description: Whether this individual tab's label should be centered inside the tab. Defaults to false.
  - class
  - name: color
    type: TEXT
    description: Color of the tab's text, and of its underline when it is active. See preview.tabler.io/colors.html
      for a list of available colors.
  - name: description
    type: TEXT
    description: Description of the tab. This is displayed when the user hovers over the tab.
  - name: icon
    type: TEXT
    description: Name of the icon to display on the tab. See tabler-icons.io for a list of available icons.
  - id
  - name: link
    type: TEXT
    description: Link to the page to display when the tab is clicked. By default, the link refers to the current
      page, with a 'tab' parameter set to the tab's title and hash set to the id (if passed) - this brings
      us back to the location of the tab after submission.
  - name: title
    type: TEXT
    required: true
    description: Text to display on the tab. If link is not set, the link will be the current page with a '$tab'
      parameter set to the tab's title. If 'id' is set, the page will be scrolled to the tab.
---

Build a tabbed interface, with each tab being a link to a page. Each tab can be in two states: active or inactive.
