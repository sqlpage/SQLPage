---
icon: table
category: display-data
summary: Turn query results into a sortable, searchable table.
top:
  - name: align_center
    type: TEXT
    description: Name of a column the contents of which should be center-aligned. This argument can be repeated
      multiple times to align multiple columns to the center.
  - name: align_right
    type: TEXT
    description: Name of a column the contents of which should be right-aligned. This argument can be repeated
      multiple times to align multiple columns to the right. Introduced in v0.15.0.
  - name: border
    type: BOOLEAN
    description: Whether to draw borders on all sides of the table and cells.
  - class
  - name: currency
    type: TEXT
    description: The ISO 4217 currency code (e.g., USD, EUR, GBP, etc.) to use when formatting monetary values.
  - name: custom_actions
    type: JSON
    description: 'If set, a column of custom action buttons will be added to each row. The value of this property
      should be a JSON array of objects, each object defining a button with the following properties: `name`
      (the text to display on the button), `icon` (the tabler icon name or image link to display on the button),
      `link` (the URL to navigate to when the button is clicked, possibly containing the `{id}` placeholder
      that will be replaced by the value of the `_sqlpage_id` property for that row), and `tooltip` (optional
      text to display when hovering over the button). Added in v0.39.0'
  - name: delete_url
    type: TEXT
    description: If set, a delete button will be added to each row. The value of this property should be a
      URL, possibly containing the `{id}` placeholder that will be replaced by the value of the `_sqlpage_id`
      property for that row. Clicking the delete button will take the user to that URL. Added in v0.39.0
  - name: description
    type: TEXT
    description: Description of the table contents. Helps users with screen readers to find a table and understand
      what it’s about.
  - name: edit_url
    type: TEXT
    description: If set, an edit button will be added to each row. The value of this property should be a URL,
      possibly containing the `{id}` placeholder that will be replaced by the value of the `_sqlpage_id` property
      for that row. Clicking the edit button will take the user to that URL. Added in v0.39.0
  - name: empty_description
    type: TEXT
    description: Text to display if the table does not contain any row. Defaults to "no data".
  - name: freeze_columns
    type: BOOLEAN
    description: Whether to freeze the leftmost column of the table.
  - name: freeze_footers
    type: BOOLEAN
    description: Whether to freeze the footer (bottom row) of the table, only works if that row has the `_sqlpage_footer`
      property applied to it.
  - name: freeze_headers
    type: BOOLEAN
    description: Whether to freeze the top row of the table.
  - name: hover
    type: BOOLEAN
    description: Whether to enable a hover state on table rows.
  - name: icon
    type: TEXT
    description: Set this to the name of a column whose content should be interpreted as a tabler icon name.
      Used to display icons in the table. This argument can be repeated multiple times to intepret multiple
      columns as icons. Introduced in v0.8.0.
  - id
  - name: initial_search_value
    type: TEXT
    description: Pre-fills the search bar used to filter the table. The user will still be able to edit the
      value to display table rows that will initially be filtered out.
  - name: markdown
    type: TEXT
    description: Set this to the name of a column whose content should be interpreted as markdown . Used to
      display rich text with links in the table. This argument can be repeated multiple times to intepret multiple
      columns as markdown.
  - name: money
    type: TEXT
    description: Name of a numeric column whose values should be displayed as currency amounts, in the currency
      defined by the `currency` property. This argument can be repeated multiple times.
  - name: monospace
    type: TEXT
    description: Name of a column the contents of which should be displayed in monospace. This argument can
      be repeated multiple times to display multiple columns in monospace. Introduced in v0.32.1.
  - name: number_format_digits
    type: INTEGER
    description: Maximum number of decimal digits to display for numeric values.
  - name: number_format_locale
    type: TEXT
    description: A BCP 47 language tag (e.g. "de-DE", "en-IN") deciding how numbers and monetary values are
      formatted. Defaults to the locale of the visitor's browser.
  - name: overflow
    type: BOOLEAN
    description: Whether to to let "wide" tables overflow across the right border and enable browser-based
      horizontal scrolling.
  - name: raw_numbers
    type: TEXT
    description: Name of a column whose values are numeric, but should be displayed as raw numbers without
      any formatting (no thousands separators, decimal separator is always a dot). This argument can be repeated
      multiple times.
  - name: search
    type: BOOLEAN
    description: Add a search bar at the top of the table, letting users easily filter table rows by value.
  - name: search_placeholder
    type: TEXT
    description: Customizes the placeholder text shown in the search input field. Replaces the default "Search..."
      with text that better describes what users should search for.
  - name: small
    type: BOOLEAN
    description: Whether to use compact table.
  - name: sort
    type: BOOLEAN
    description: Make the columns clickable to let the user sort by the value contained in the column.
  - name: striped_columns
    type: BOOLEAN
    description: Whether to add zebra-striping to any table column.
  - name: striped_rows
    type: BOOLEAN
    description: Whether to add zebra-striping to any table row.
  - name: title
    type: TEXT
    description: A title displayed above the table.
row:
  - name: _sqlpage_actions
    type: JSON
    description: 'Sets custom action buttons for this specific row in addition to any defined at the table
      level, The value of this property should be a JSON array of objects, each object defining a button with
      the following properties: `name` (the text to display on the button), `icon` (the tabler icon name or
      image link to display on the button), `link` (the URL to navigate to when the button is clicked, possibly
      containing the `{id}` placeholder that will be replaced by the value of the `_sqlpage_id` property for
      that row), and `tooltip` (optional text to display when hovering over the button). Added in v0.39.0'
  - name: _sqlpage_color
    type: COLOR
    description: Sets the background color of the row. Added in v0.8.0.
  - name: _sqlpage_css_class
    type: TEXT
    description: For advanced users. Sets a css class on the table row. Added in v0.8.0.
  - name: _sqlpage_footer
    type: BOOLEAN
    description: Sets this row as the table footer. It is recommended that this parameter is applied to the
      last row. Added in v0.34.0.
  - name: _sqlpage_id
    type: TEXT
    description: Sets the id of the html tabler row element. Allows you to make links targeting a specific
      row in a table.
---

A table with optional filtering and sorting.
Unlike most others, this component does not have a fixed set of item properties, any property that is used will be rendered directly as a column in the table.
Tables can contain rich text, including images, links, and icons. Table rows can be styled with a background color, and the table can be made striped, hoverable, and bordered.

Advanced users can apply custom styles to table columns using a CSS class with the same name as the column, and to table rows using the `_sqlpage_css_class` property.
