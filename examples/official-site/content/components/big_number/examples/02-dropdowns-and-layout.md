---
rows:
  - component: big_number
    columns: 3
    id: colorfull_dashboard
  - title: Users
    value: 1,234
    color: red
    title_link: '#users'
    title_link_new_tab: false
    value_link: '#users_details'
    value_link_new_tab: true
  - title: Orders
    value: 56
    color: green
    title_link: '#orders'
    title_link_new_tab: true
  - title: Revenue
    value: 9,876
    unit: €
    color: blue
    change_percent: -7
    dropdown_item:
      - label: This week
        link: ?days=7&component=big_number#colorfull_dashboard
      - label: This month
        link: ?days=30&component=big_number#colorfull_dashboard
      - label: This quarter
        link: ?days=90&component=big_number#colorfull_dashboard
---

Big numbers with dropdowns and customized layout
