---
icon: filter
since: 0.46.0
category: navigation
summary: Let readers choose filters for a dataset.
top:
  - name: all_active
    type: BOOLEAN
    description: Whether the link that clears the filter is active. Defaults to false.
  - name: all_link
    type: URL
    description: URL that clears the filter. If omitted, no All link is displayed.
  - name: all_title
    type: TEXT
    description: Text for the link that clears the filter. Defaults to "ALL".
  - name: center
    type: BOOLEAN
    description: Whether to center the facet selector
  - class
  - name: compact
    type: BOOLEAN
    description: Displays the facets in a dropdown instead of links.
  - name: description
    type: TEXT
    description: The facet category label.
  - name: dropdown_title
    type: TEXT
    description: Text shown on the compact-mode dropdown button when no facet is selected. Defaults to `Choose
      facet`.
  - id
  - name: selected_facet
    type: TEXT
    description: An alias for `dropdown_title`.
row:
  - name: active
    type: BOOLEAN
    description: Whether the link is active or not. Defaults to false.
  - name: link
    type: URL
    required: true
    description: URL for the facet.
  - name: title
    type: TEXT
    required: true
    description: Facet title.
---

Navigation links for filtering a dataset by a category, status, owner, or other attribute.

This component only renders the filter navigation. **Your SQL query is responsible for filtering the data** based on the URL parameter selected by the user.

Use it alongside a [table](?component=table), [list](?component=list), or [card](?component=card). The links use GET parameters, so the selected filter can be bookmarked and shared.

Set `compact` to display the facets in a dropdown, which is useful for a moderate number of choices or on narrow screens. Use `selected_facet` to show the active facet in the closed dropdown. `all_link` and `all_title` add a link that clears the filter.

The following portable pattern filters a table by category. `sqlpage.link` preserves the current page path while safely generating the URL.

```sql
select 'table' as component;
select title, category
from my_table
where $category is null or category = $category;

select 'facet' as component,
    'Category' as description,
    sqlpage.link(sqlpage.path(), json_object('category', null)) as all_link,
    $category is null as all_active;
select category as title,
    sqlpage.link(sqlpage.path(), json_object('category', category)) as link,
    category = $category as active
from my_table
group by category
order by category;
```
