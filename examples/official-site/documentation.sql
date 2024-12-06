-- ensure that the component exists and do not render this page if it does not
select 'redirect' as component, sqlpage.link('component.sql', json_object('component', $component)) as link
where $component is not null;

-- This line, at the top of the page, tells web browsers to keep the page locally in cache once they have it.
select 'http_header' as component, 'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', coalesce($component || ' - ', '') || 'SQLPage Documentation'
)) as properties
FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, format('SQLPage v%s documentation', sqlpage.version()) as title;
select '
If you are completely new to SQLPage, you should start by reading the [get started tutorial](get%20started.sql),
which will guide you through the process of creating your first SQLPage application.

Building an application with SQLPage is quite simple.
To create a new web page, just create a new SQL file. 
For each SELECT statement that you write, the data it returns will be analyzed and rendered to the user.
The two most important concepts in SQLPage are **components** and **parameters**.

 - **components** are small user interface elements that you can use to display your data in a certain way.
 - *top-level* **parameters** are the properties of these components, allowing you to customize their appearance and behavior.
 - *row-level* **parameters** constitute the data that you want to display in the components.

To select a component and set its top-level properties, you write the following SQL statement: 

```sql
SELECT ''component_name'' AS component, ''my value'' AS top_level_parameter_1;
```

Then, you can set its row-level parameters by writing a second SELECT statement:

```sql
SELECT my_column_1 AS row_level_parameter_1, my_column_2 AS row_level_parameter_2 FROM my_table;
```

This page documents all the components provided by default in SQLPage and their parameters.
Use this as a reference when building your SQL application.
If at any point you need help, you can ask for it on the [SQLPage forum](https://github.com/sqlpage/SQLPage/discussions).

If you know some [HTML](https://developer.mozilla.org/en-US/docs/Learn/Getting_started_with_the_web/HTML_basics),
you can also easily [create your own components for your application](./custom_components.sql).
' as contents_md;

select 'list' as component, 'components' as title;
select
    name as title,
    description,
    icon,
    sqlpage.link('component.sql', json_object('component', name)) as link
from component
order by name;
