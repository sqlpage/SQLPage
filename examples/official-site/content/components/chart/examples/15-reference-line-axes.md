---
rows:
  - component: chart
    title: Disk usage
    type: bar
    horizontal: true
    ymax: 100
    color: azure
    labels: true
  - yline: 90
    label: full
    color: red
  - xline: db-1
    label: watched
    color: purple
  - x: backup-1
    y: 41
  - x: web-2
    y: 63
  - x: db-1
    y: 88
  - x: web-1
    y: 96
---

## Reference lines follow their axis

A reference belongs to the column it is written in, not to a direction on the
screen. `yline` always marks a value of `y`, and `xline` a position on `x`,
whichever way round the chart is drawn. A `horizontal` bar chart runs its y axis
from left to right, so a `yline` is drawn down the chart and an `xline` picks out
one of the bars.

```sql
select 'chart' as component, 'bar' as type, true as horizontal, 100 as ymax;
select 90 as yline, 'full' as label, 'red' as color;
select 'db-1' as xline, 'watched' as label, 'purple' as color;
select host as x, percent_used as y from disks order by percent_used;
```

A `pie` has no axes and ignores reference lines, and on a `heatmap`, whose y axis
holds the names of the series, only `xline` has a meaning.
