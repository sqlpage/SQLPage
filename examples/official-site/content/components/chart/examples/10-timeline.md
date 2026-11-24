---
rows:
  - component: chart
    title: Project Timeline
    type: rangeBar
    time: true
    color:
      - teal
      - cyan
    labels: true
    xmin: '2021-12-28'
    xmax: '2022-01-04'
  - series: Phase 1
    label: Operations
    value:
      - '2021-12-29'
      - '2022-01-02'
    link: /examples/chart.sql?phase=1
  - series: Phase 2
    label: Operations
    value:
      - '2022-01-03'
      - '2022-01-04'
  - series: Yearly maintenance
    label: Maintenance
    value:
      - '2022-01-01'
      - '2022-01-03'
---

A timeline displaying events with a start and an end date. A data row can include a `link` to open when its bar or tooltip title is clicked.
