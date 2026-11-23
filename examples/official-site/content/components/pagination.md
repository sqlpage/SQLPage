---
icon: sailboat-2
since: 0.40.0
category: navigation
summary: Link to other pages of a query result.
top:
  - name: circle
    type: BOOLEAN
    description: Whether to use circle version of the pagination.
  - class
  - name: first_disabled
    type: BOOLEAN
    description: disables the button to go to the first page.
  - name: first_link
    type: URL
    description: A target URL to which the user should be directed to get to the first page. If none, the link
      is not displayed.
  - name: first_title
    type: TEXT
    description: The text displayed on the button to go to the first page.
  - id
  - name: last_disabled
    type: BOOLEAN
    description: disables the button to go to the last page.
  - name: last_link
    type: URL
    description: A target URL to which the user should be directed to get to the last page. If none, the link
      is not displayed.
  - name: last_title
    type: TEXT
    description: The text displayed on the button to go to the last page.
  - name: next_disabled
    type: BOOLEAN
    description: Disables the button to go to the next page.
  - name: next_link
    type: URL
    description: A target URL to which the user should be directed to get to the next page. If none, the link
      is not displayed.
  - name: next_title
    type: TEXT
    description: The text displayed on the button to go to the next page.
  - name: outline
    type: BOOLEAN
    description: Whether to use outline version of the pagination.
  - name: previous_disabled
    type: BOOLEAN
    description: disables the button to go to the previous page.
  - name: previous_link
    type: URL
    description: A target URL to which the user should be directed to get to the previous page. If none, the
      link is not displayed.
  - name: previous_title
    type: TEXT
    description: The text displayed on the button to go to the previous page.
row:
  - name: active
    type: BOOLEAN
    description: Whether the link is active or not. Defaults to false.
  - name: contents
    type: INTEGER
    required: true
    description: Page number.
  - name: link
    type: URL
    description: A target URL to which the user should be redirected to view the requested page of data.
  - name: offset
    type: BOOLEAN
    description: Whether to use offset to show only a few pages at a time. Usefull if the count of pages is
      too large. Defaults to false
---

Navigation links to go to the first, previous, next, or last page of a dataset.
Useful when data is divided into pages, each containing a fixed number of rows.

This component only handles the display of pagination.
**Your sql queries are responsible for filtering data** based on the page number passed as a URL parameter.

This component is typically used in conjunction with a [table](?component=table),
[list](?component=list), or [card](?component=card) component.

The pagination component displays navigation buttons (first, previous, next, last) customizable with text or icons.

For large numbers of pages, an offset can limit the visible page links.

A minimal example of a SQL query that uses the pagination would be:
```sql
select 'table' as component;
select * from my_table limit 100 offset $offset;

select 'pagination' as component;
with recursive pages as (
    select 0 as offset
    union all
    select offset + 100 from pages
    where offset + 100 < (select count(*) from my_table)
)
select
    (offset/100+1) as contents,
    sqlpage.link(sqlpage.path(), json_object('offset', offset)) as link,
    offset = coalesce(cast($offset as integer), 0) as active
from pages;
```

For more advanced usage, the [pagination guide](blog.sql?post=How+to+use+the+pagination+component) provides a complete tutorial.
