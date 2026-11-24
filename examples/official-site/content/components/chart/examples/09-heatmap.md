---
rows:
  - component: chart
    title: Survey Results
    type: heatmap
    ytitle: Database managemet system
    xtitle: Year
    color:
      - purple
      - purple
      - purple
  - series: PostgreSQL
    x: '2000'
    y: 48
  - series: SQLite
    x: '2000'
    y: 44
  - series: MySQL
    x: '2000'
    y: 78
  - series: PostgreSQL
    x: '2010'
    y: 65
  - series: SQLite
    x: '2010'
    y: 62
  - series: MySQL
    x: '2010'
    y: 83
  - series: PostgreSQL
    x: '2020'
    y: 73
  - series: SQLite
    x: '2020'
    y: 38
  - series: MySQL
    x: '2020'
    y: 87
---

## Heatmaps

You can build heatmaps using the `heatmap` top-level property.

The data format follows the [apexcharts heatmap format](https://apexcharts.com/angular-chart-demos/heatmap-charts/basic/),
where each series is represented as a line in the chart:
 - The `x` property of each item will be used as the x-axis value.
 - The `series` property of each item will be used as the y-axis value.
 - The `y` property of each item will be used as the value to display in the heatmap

The `color` property sets the color of each series separately, in order.
