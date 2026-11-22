---
rows:
  - component: pagination
    first_title: First
    last_title: Last
    previous_title: Previous
    next_title: Next
    first_link: ?component=pagination
    first_disabled: true
    previous_link: ?component=pagination
    previous_disabled: true
    next_link: '#?page=2'
    last_link: '#?page=3'
  - contents: 1
    link: ?component=pagination&page=1
    active: true
  - contents: 2
    link: ?component=pagination&page=2
  - contents: 3
    link: ?component=pagination&page=3
---

Instead of using icons, you can apply text to the navigation links.
