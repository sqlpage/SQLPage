---
icon: chevrons-down
since: 0.29.0
category: page-layout
summary: Let readers expand individual sections to see their contents.
top:
  - name: class
    type: TEXT
    description: CSS class(es) to add to the foldable container. When set at the top level, applies to the
      entire foldable component.
  - name: id
    type: TEXT
    description: ID attribute added to the container in HTML. Used for targeting through CSS or for scrolling
      via links. When set at the top level, applies to the entire foldable component.
row:
  - name: class
    type: TEXT
    description: CSS class(es) to add to individual foldable items.
  - name: description
    type: TEXT
    description: Plain text description of the item, displayed when expanded.
  - name: description_md
    type: TEXT
    description: Markdown description of the item, displayed when expanded.
  - name: expanded
    type: BOOLEAN
    description: If set to TRUE, the foldable item starts in an expanded state. Defaults FALSE
  - name: id
    type: TEXT
    description: ID attribute added to individual foldable items. Used for targeting through CSS or for scrolling
      via links.
  - name: title
    type: TEXT
    description: Title of the foldable item, displayed on the button.
---

A foldable list of elements which can be expanded individually.
