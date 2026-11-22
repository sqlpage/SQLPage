---
rows:
  - component: chart
    title: Request latency
    type: area
    time: true
    ytitle: ms
    color: blue-lt
    marker: 3
  - xline: '2024-05-01T10:00:00Z'
    label: deploy
    color: green
  - xline: '2024-05-01T11:30:00Z'
    xline_end: '2024-05-01T13:00:00Z'
    label: incident
    color: red
  - x: '2024-05-01T08:00:00Z'
    y: 120
  - x: '2024-05-01T09:00:00Z'
    y: 134
  - x: '2024-05-01T10:00:00Z'
    y: 128
  - x: '2024-05-01T11:00:00Z'
    y: 141
  - x: '2024-05-01T12:00:00Z'
    y: 512
  - x: '2024-05-01T13:00:00Z'
    y: 470
  - x: '2024-05-01T14:00:00Z'
    y: 156
  - x: '2024-05-01T15:00:00Z'
    y: 133
---

## Marking events

`xline` is the counterpart of `yline`: it marks a position on the x axis instead
of a value on the y axis. On its own it marks a moment, like a deployment.
With `xline_end`, it covers everything in between, like an incident or a
maintenance window. A single query can draw a whole log of them:

```sql
select started_at as xline, ended_at as xline_end, summary as label,
    case severity when 'outage' then 'red' else 'orange' end as color
from incidents where started_at > $since;
```

When `time` is set, an `xline` is a date or a timestamp, written like the `x` of
a data point. On a chart with text labels on the x axis, it is one of those labels.
