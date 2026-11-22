---
rows:
  - component: card
    title: A dashboard with multiple graphs on the same line
    columns: 2
  - embed: /examples/chart.sql?color=green&n=42&_sqlpage_embed
  - embed: /examples/chart.sql?_sqlpage_embed
---

## Multiple charts on the same line

You can create information-dense dashboards by using the [card component](?component=card#component)
to put multiple charts on the same line.

For this, create one sql file per visualization you want to show,
and set the `embed` attribute of the [card](?component=card#component) component
to the path of the file you want to include, followed by `?_sqlpage_embed`.
