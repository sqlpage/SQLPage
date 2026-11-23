---
rows:
  - component: chart
    title: Accounts expiring soon
    type: bar
    horizontal: true
    labels: true
    show_legend: false
  - label: 30 days
    value: 100
    color: red
  - label: 60 days
    value: 200
    color: orange
  - label: 90 days
    value: 300
    color: green
---

## Coloring a single value

A data row can carry its own `color`, to paint the one bar, slice or point it
plots. Use it when the color says something the axes do not: a threshold
crossed, a status, the one category the reader should look at first.

```sql
select 'chart' as component, 'bar' as type, true as horizontal,
    true as labels, false as show_legend;
select
    window_label as label,
    accounts as value,
    case when days <= 30 then 'red' when days <= 60 then 'orange' else 'green' end as color
from expiring_accounts order by days;
```

A row color takes precedence over the color of its series. On a `line` or an
`area` chart it paints the marker of the point, so set `marker` for it to show.
A `heatmap` shades its cells from their own value and ignores it.
