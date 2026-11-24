---
rows:
  - component: chart
    title: Power draw
    type: area
    stacked: true
    time: true
    ytitle: watts
    color:
      - blue
      - teal
    marker: 4
  - series: CPU
    x: '2024-03-01T10:00:00Z'
    value: 45
  - series: CPU
    x: '2024-03-01T10:15:00Z'
    value: 52
  - series: CPU
    x: '2024-03-01T10:30:00Z'
    value: 48
  - series: CPU
    x: '2024-03-01T10:45:00Z'
    value: 44
  - series: GPU
    x: '2024-03-01T10:15:00Z'
    value: 120
  - series: GPU
    x: '2024-03-01T10:30:00Z'
    value: 140
---

A stacked area chart, showing how each series contributes to a total.
The `stacked` property also works with the `line` and `bar` chart types.

Series are aligned on their `x` values, and a series that has no value for a given `x` counts as zero there:
below, the graphics card draws no power outside of the render.
If a missing value does not mean zero in your data, make all the series share the same `x` values,
for instance by rounding timestamps to a common interval.
