---
rows:
  - component: table
  - name: USS Enterprise (NCC-1701)
    class: Constitution
  - name: USS Exeter (NCC-1672)
    class: Galaxy
  - name: USS Exeter (NCC-1672)
    class: Constitution
  - component: facet
    description: Classes
    all_link: ?component=facet
    all_title: All
    all_active: false
  - title: Constitution
    link: ?component=facet&class=Constitution
    active: true
  - title: Galaxy
    link: ?component=facet&class=Galaxy
---

A category selector with an All link. In an application, the SQL pattern above uses the chosen URL parameter to filter the displayed data.
