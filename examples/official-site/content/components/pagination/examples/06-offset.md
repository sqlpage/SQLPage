---
rows:
  - component: pagination
    first_link: '#?page=1'
    previous_link: '#?page=3'
    next_link: '#?page=4'
    last_link: '#?page=99'
  - contents: 1
    link: ?component=pagination&page=1
  - contents: 2
    link: ?component=pagination&page=2
  - contents: 3
    link: ?component=pagination&page=3
  - contents: 4
    link: ?component=pagination&page=4
    active: true
  - contents: 5
    link: ?component=pagination&page=5
  - contents: 6
    link: ?component=pagination&page=6
  - offset: true
  - contents: 99
    link: ?component=pagination&page=99
---

If you have a large number of pages to display, you can use an offset to represent a group of pages.
