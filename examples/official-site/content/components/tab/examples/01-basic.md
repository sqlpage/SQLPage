---
rows:
  - component: tab
  - title: This tab does not exist
    active: true
    link: ?component=tab&tab=tab_1
  - title: I am not a true tab
    link: ?component=tab&tab=tab_2
  - title: Do not click here
    link: ?component=tab&tab=tab_3
---

This example shows a very basic set of three tabs. The first tab is active. You could use this at the top of a page for easy navigation.

To implement contents that change based on the active tab, use the `tab` parameter in the page query string.
For example, if the page is `/my-page.sql`, then the first tab will have a link of `/my-page.sql?tab=My+First+tab`.

You could then for instance display contents coming from the database based on the value of the `tab` parameter.
For instance: `SELECT 'text' AS component, contents_md FROM my_page_contents WHERE tab = $tab`.
Or you could write different queries for different tabs and use the `$tab` parameter with a static value in a where clause to switch between tabs:

```sql
select 'tab' as component;
select 'Projects' as title, $tab = 'Projects' as active;
select 'Tasks'    as title, $tab = 'Tasks'    as active;

select 'table' as component;

select * from my_projects where $tab = 'Projects';
select * from my_tasks    where $tab = 'Tasks';
```

Note that the example below is completely static, and does not use the `tab` parameter to actually switch between tabs.
View the [dynamic tabs example](/examples/tabs/).
