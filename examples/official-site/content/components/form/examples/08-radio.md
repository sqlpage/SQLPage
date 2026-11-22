---
rows:
  - component: form
    method: GET
  - name: component
    type: radio
    value: form
    description: Read user input in SQL
    label: Form
  - name: component
    type: radio
    value: map
    checked: true
    description: Display a map based on database data
    label: Map
  - name: component
    type: radio
    value: chart
    description: Interactive plots of SQL query results
    label: Chart
---

This example illustrates the use of the `radio` type.
The `name` parameter is used to group the radio buttons together.
The `value` parameter is used to set the value that will be submitted when the user selects the radio button.
The `label` parameter is used to display a friendly name for the radio button.
The `description` parameter is used to display a helper text near the radio button.

We could also save all the options in a database table, and then run a simple query like

```sql
SELECT 'form' AS component;
SELECT
    'radio' as type,
    'db' as name,
    option_name as label,
    option_id as value
FROM my_options;
```

In this example, depending on what the user clicks, the page will be reloaded with a the variable `$component` set to the string "form", "map", or "chart".
