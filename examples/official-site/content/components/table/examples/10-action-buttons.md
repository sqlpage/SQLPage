---
rows:
  - component: table
    edit_url: /examples/show_variables.sql?action=edit&update_id={id}
    delete_url: /examples/show_variables.sql?action=delete&delete_id={id}
    custom_actions:
      name: history
      tooltip: View Standard History
      link: /examples/show_variables.sql?action=history&standard_id={id}
      icon: history
  - name: CalStd
    vendor: PharmaCo
    Product: P1234
    lot number: T23523
    status: Available
    expires on: '2026-10-13'
    _sqlpage_id: 32
    _sqlpage_actions:
      - name: View PDF
        tooltip: View Presentation
        link: https://sql-page.com/pgconf/2024-sqlpage-badass.pdf
        icon: file-type-pdf
      - name: Action
        tooltip: Set In Use
        link: /examples/show_variables.sql?action=set_in_use&standard_id=32
        icon: caret-right
  - name: CalStd
    vendor: PharmaCo
    Product: P1234
    lot number: T2352
    status: In Use
    expires on: '2026-10-14'
    _sqlpage_id: 33
    _sqlpage_actions:
      - name: View PDF
        tooltip: View Presentation
        link: https://sql-page.com/pgconf/2024-sqlpage-badass.pdf
        icon: file-type-pdf
      - name: Action
        tooltip: Retire Standard
        link: /examples/show_variables.sql?action=retire&standard_id=33
        icon: test-pipe-off
  - name: CalStd
    vendor: PharmaCo
    Product: P1234
    lot number: A123
    status: Discarded
    expires on: '2026-09-30'
    _sqlpage_id: 31
    _sqlpage_actions:
      - name: View PDF
        tooltip: View Presentation
        link: https://sql-page.com/pgconf/2024-sqlpage-badass.pdf
        icon: file-type-pdf
      - name: Action
---

## Using Action Buttons in a table.

### Preset Actions: `edit_url` & `delete_url`
Since edit and delete are common actions, the `table` component has dedicated `edit_url` and `delete_url` properties to add buttons for these actions.
The value of these properties should be a URL, containing the `{id}` placeholder that will be replaced by the value of the `_sqlpage_id` property for that row.

### Column with fixed action buttons

You may want to add custom action buttons to your table rows, for instance to view details, download a file, or perform a custom operation.
For this, the `table` component has a `custom_actions` top-level property that lets you define a column of buttons, each button defined by a name, an icon, a link, and an optional tooltip.

### Column with variable action buttons

The `table` component also supports the row level `_sqlpage_actions` column in your data table.
This is helpful if you want a more complex logic, for instance to disable a button on some rows, or to change the link or icon based on the row data.

> WARNING!
> If the number of array items in `_sqlpage_actions` is not consistent across all rows, the table may not render correctly.
> You can leave blank spaces by including an object with only the `name` property.

The table has a column of buttons, each button defined by the `custom_actions` column at the table level, and by the `_sqlpage_actions` property at the row level.

### `custom_actions` & `_sqlpage_actions` JSON properties.

Each button is defined by the following properties:
* `name`: sets the column header and the tooltip if no tooltip is provided,
* `tooltip`: text to display when hovering over the button,
* `link`: the URL to navigate to when the button is clicked, possibly containing the `{id}` placeholder that will be replaced by the value of the `_sqlpage_id` property for that row,
* `icon`: the tabler icon name or image link to display on the button

### Example using all of the above
