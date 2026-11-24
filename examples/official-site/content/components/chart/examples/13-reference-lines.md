---
rows:
  - component: chart
    title: CPU temperature
    type: line
    time: true
    ytitle: °C
    ymax: 100
    color: azure
    marker: 4
  - yline: 70
    label: target
    color: green
  - yline: 90
    yline_end: 100
    label: throttling
    color: red
  - x: '2024-05-01T08:00:00Z'
    y: 52
  - x: '2024-05-01T09:00:00Z'
    y: 58
  - x: '2024-05-01T10:00:00Z'
    y: 71
  - x: '2024-05-01T11:00:00Z'
    y: 83
  - x: '2024-05-01T12:00:00Z'
    y: 94
  - x: '2024-05-01T13:00:00Z'
    y: 76
  - x: '2024-05-01T14:00:00Z'
    y: 63
---

## Reference lines

A row with a `yline` is not plotted as a data point, but drawn as a line across
the whole chart, at that value of the y axis. Use it for the limit that the data
should be read against: a disk quota, an alarm threshold, a service level
objective. Add `yline_end` to make it a band instead of a line.

Reference lines are rows, so they come from a query like everything else,
and a chart can have as many of them as the query returns:

```sql
select 'chart' as component, 'CPU temperature' as title, true as time, 100 as ymax;
select celsius as yline, name as label, color as color from thresholds;
select measured_at as x, celsius as y from readings order by measured_at;
```

They are drawn as annotations rather than as an extra series, so they are not
added to the total of a `stacked` chart, and are not filled in an `area` chart.

A line outside of the y axis is not drawn, and does not stretch the axis to fit,
so set `ymax` when the limit is above the data.
