---
icon: cursor-text
category: forms-and-feedback
summary: Collect input with text fields, file uploads, and other controls.
top:
  - name: action
    type: TEXT
    description: An optional link to a target page that will handle the results of the form. By default the
      target page is the current page with the id of the form (if passed) used as hash - this will bring us
      back to the location of the form after submission. Setting it to the name of a different sql file will
      load that file when the user submits the form.
  - name: auto_submit
    type: BOOLEAN
    description: Automatically submit the form when the user changes any of its fields, and remove the validation
      button.
  - class
  - name: enctype
    type: TEXT
    description: |2

      When ``method="post"``, this specifies how the form-data should be encoded
      when submitting it to the server.
  - name: id
    type: TEXT
    description: A unique identifier for the form, which can then be used to validate the form from a button
      outside of the form.
  - name: method
    type: TEXT
    description: Set this to 'GET' to pass the form contents directly as URL parameters. If the user enters
      a value v in a field named x, submitting the form will load target.sql?x=v. If target.sql contains SELECT
      $x, it will display the value v.
  - name: narrow
    type: BOOLEAN
    description: Whether to trim the horizontal padding between the icon and the label of the validation and
      reset buttons.
  - name: reset
    type: TEXT
    description: The text to display in the button at the bottom of the form that resets the form to its original
      state. Omit this property not to show a reset button at all.
  - name: reset_color
    type: COLOR
    description: The color of the button at the bottom of the form that resets the form to its original state.
      Omit this property to use the default color.
  - name: reset_icon
    type: ICON
    description: Name of an icon to be displayed on the left side of the reset button.
  - name: reset_outline
    type: COLOR
    description: A color to outline the reset button.
  - name: reset_shape
    type: TEXT
    description: The shape of the reset button (e.g., pill, square).
  - name: reset_size
    type: TEXT
    description: The size of the reset button (e.g., sm, lg).
  - name: title
    type: TEXT
    description: A name to display at the top of the form. It will be displayed in a larger font size at the
      top of the form.
  - name: validate
    type: TEXT
    description: The text to display in the button at the bottom of the form that submits the values. Omit
      this property to let the browser display the default form validation text, or set it to the empty string
      to remove the button completely.
  - name: validate_color
    type: COLOR
    description: The color of the button at the bottom of the form that submits the values. Omit this property
      to use the default color.
  - name: validate_icon
    type: ICON
    description: Name of an icon to be displayed on the left side of the submit button.
  - name: validate_outline
    type: COLOR
    description: A color to outline the validation button.
  - name: validate_shape
    type: TEXT
    description: The shape of the validation button (e.g., pill, square).
  - name: validate_size
    type: TEXT
    description: The size of the validation button (e.g., sm, lg).
row:
  - name: accept
    type: TEXT
    description: Used only for inputs of type "file". A comma-separated list of the file types the user can
      pick, given as extensions (".pdf,.png") or as MIME types ("image/*").
  - name: autocomplete
    type: BOOLEAN
    description: Whether the browser should suggest previously entered values for this field.
  - name: autofocus
    type: BOOLEAN
    description: Automatically focus the field when the page is loaded
  - name: checked
    type: BOOLEAN
    description: Used only for checkboxes and radio buttons. Indicates whether the checkbox should appear as
      already checked.
  - name: class
    type: TEXT
    description: A CSS class to apply to the form element.
  - name: create_new
    type: BOOLEAN
    description: In a select with a dropdown (`searchable`, `dropdown` or `options_source`), this option allows
      the user to enter new values, that are not in the list of options. Works for single selects and for multiselects.
  - name: description
    type: TEXT
    description: A helper text to display near the input field.
  - name: description_md
    type: TEXT
    description: A helper text to display near the input field - formatted using markdown.
  - name: disabled
    type: BOOLEAN
    description: Makes the field non-editable, non-focusable, and not submitted with the form. Use readonly
      instead for simple non-editable fields.
  - name: dropdown
    type: BOOLEAN
    description: An alias for "searchable".
  - name: empty_option
    type: TEXT
    description: Only for inputs of type `select`. Adds an empty option with the given label before the ones
      defined in `options`. Useful when generating other options from a database table.
  - name: formaction
    type: TEXT
    description: When type is "submit", this specifies the URL of the file that will handle the form submission.
      Useful when you need multiple submit buttons.
  - name: formenctype
    type: TEXT
    description: |2

      When ``type`` is ``submit`` or ``image``, this specifies how the form-data
      should be encoded when submitting it to the server.

      Takes precedence over any ``enctype`` set on the ``form`` element.

      NOTE: when a ``file`` type input is present, then ``formenctype="multipart/form-data"``
      is automatically applied to the default validate button.
  - name: formmethod
    type: TEXT
    description: When type is "submit", the HTTP method to submit the form with. Takes precedence over the
      form's own "method".
  - name: formnovalidate
    type: BOOLEAN
    description: When type is "submit", skips the browser's validation of the other fields when this button
      submits the form.
  - name: formtarget
    type: TEXT
    description: 'When type is "submit", where to display the response. Takes the same values as a link target:
      "_blank" to open a new tab, "_self" to stay in the same one.'
  - name: id
    type: TEXT
    description: A unique identifier for the input, which can then be used to select and manage the field with
      Javascript code. Usefull for advanced using as setting client side event listeners, interactive control
      of input field (disabled, visibility, read only, e.g.) and AJAX requests.
  - name: label
    type: TEXT
    description: A friendly name for the text field to show to the user.
  - name: list
    type: TEXT
    description: The id of a datalist element holding the suggestions to show for this field. The datalist
      itself has to be rendered by the "html" component.
  - name: max
    type: REAL
    description: The maximum value to accept for an input of type number
  - name: maxlength
    type: INTEGER
    description: Maximum length of text allowed in the field.
  - name: min
    type: REAL
    description: The minimum value to accept for an input of type number
  - name: minlength
    type: INTEGER
    description: Minimum length of text allowed in the field.
  - name: multiple
    type: BOOLEAN
    description: 'For select elements, indicates that multiple elements can be selected simultaneously. When
      using multiple, you should add square brackets after the variable name: ''my_variable[]'' as name. For
      file inputs, lets the user select several files at once: save them all with [sqlpage.persist_uploaded_files](functions.sql?function=persist_uploaded_files).'
  - name: name
    type: TEXT
    required: true
    description: The name of the input field, that you can use in the target page to get the value the user
      entered for the field.
  - name: options
    type: JSON
    description: A json array of objects containing the label and value of initial options of a select field.
      Used only when type=select. JSON objects in the array can contain the properties "label", "value" and
      "selected".
  - name: options_source
    type: URL
    description: Only for inputs of type `select`. URL of a SQL file that returns JSON search results for the
      dropdown. The SQL file receives the search text as `$search` and must return an array of objects with
      exactly `label` and `value`. Use `options` for options that should be available before the user searches.
  - name: pattern
    type: TEXT
    description: A regular expression that the value must match. For instance, [0-9]{3} will only accept 3
      digits.
  - name: placeholder
    type: TEXT
    description: A placeholder text that will be shown in the field when is is empty.
  - name: prefix
    type: TEXT
    description: Text to display on the left side of the input field, on the same line.
  - name: prefix_icon
    type: ICON
    description: Icon to display on the left side of the input field, on the same line.
  - name: readonly
    type: BOOLEAN
    description: Set to true to prevent the user from modifying the value of the input field.
  - name: required
    type: BOOLEAN
    description: Set this to true to prevent the form contents from being sent if this field is left empty
      by the user.
  - name: rows
    type: INTEGER
    description: Number of rows to display for a textarea. Defaults to 3.
  - name: searchable
    type: BOOLEAN
    description: For select and multiple-select elements, displays them with a nice dropdown that allows searching
      for options.
  - name: step
    type: REAL
    description: The increment of values in an input of type number. Set to 1 to allow only integers.
  - name: suffix
    type: TEXT
    description: Short text to display after th input, on the same line. Useful to add units or a currency
      symbol to an input.
  - name: type
    type: TEXT
    description: 'Declares input control behavior and expected format. All HTML input types are supported (text,
      number, date, file, checkbox, radio, hidden, ...). SQLPage adds some custom types: textarea, switch,
      header. text by default. See https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/input#input_types'
  - name: value
    type: TEXT
    description: A default value that will already be present in the field when the user loads the page.
  - name: width
    type: INTEGER
    description: Width of the form field, between 1 and 12.
---

## Building forms in SQL

The form component will display a series of input fields of various types, that can be filled in by the user.
When the user submits the form, the data is posted to an SQL file specified in the `action` property.

### Handling submitted values

The receiving SQL page will be able to handle the data,
and insert it into the database, use it to perform a search, format it, update existing data, etc.

A value in a field named "x" will be available as `:x` in the SQL query of the target page.

### Form projects

 - [A multi-step form](https://github.com/sqlpage/SQLPage/tree/main/examples/forms-with-multiple-steps), guiding the user through a process without overwhelming them with a large form.
 - [File upload form](https://github.com/sqlpage/SQLPage/tree/main/examples/image%20gallery%20with%20user%20uploads), letting users upload images to a gallery.
 - [Rich text editor](https://github.com/sqlpage/SQLPage/tree/main/examples/rich-text-editor), letting users write text with bold, italics, links, images, etc.
 - [Master-detail form](https://github.com/sqlpage/SQLPage/tree/main/examples/master-detail-forms), to edit a list of structured items.
 - [Form with a variable number of fields](https://github.com/sqlpage/SQLPage/tree/main/examples/forms%20with%20a%20variable%20number%20of%20fields), when the fields are not known in advance.
 - [Demo of all input types](/examples/form), showing all the input types supported by SQLPage.
