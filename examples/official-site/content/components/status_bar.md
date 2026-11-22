---
icon: layout-bottombar
since: 0.49.0
category: navigation
summary: Show status messages and links in a horizontal bar.
top:
  - class
  - name: fixed
    type: BOOLEAN
    description: Keep the bar at the bottom of the screen while the page scrolls. Defaults to true. A fixed
      bar overlays page content, so leave space below your last item if needed. Set to false to place the bar
      in the page where the SQL appears.
  - id
  - name: label
    type: TEXT
    description: Accessible name for the bar. Defaults to "Status bar".
row:
  - class
  - name: color
    type: COLOR
    description: Color of the segment text and icon.
  - name: icon
    type: ICON
    description: Icon displayed before the text.
  - name: link
    type: URL
    description: URL to open when the segment is clicked.
  - name: title
    type: TEXT
    description: Text displayed in the segment.
  - name: tooltip
    type: TEXT
    description: Text shown when hovering over the segment or focusing its link. For an icon without a title,
      supply a tooltip to give the segment an accessible name.
---

Display short status messages and links in a single horizontal bar. Each row adds one segment. When there are several segments, the last one aligns to the far edge. If the segments do not all fit, scroll the bar horizontally to see the rest.
