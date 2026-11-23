---
rows:
  - component: pagination
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

The following example implements navigation links that can be enabled or disabled as needed. Since a navigation link does not appear if no link is assigned to it, you must always assign a link to display it as disabled.
