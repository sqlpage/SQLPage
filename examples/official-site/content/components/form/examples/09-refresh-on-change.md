---
rows:
  - component: form
    auto_submit: true
  - name: component
    type: select
    autocomplete: false
    options:
      - label: Form
        value: form
        selected: true
      - label: Map
        value: map
      - label: Chart
        value: chart
    description: Choose a component to view its documentation. No need to click a button, the page will be
      reloaded automatically.
    label: Component
---

### Dynamically refresh the page when the user changes the form

The form will be automatically submitted when the user changes any of its fields, and the page will be reloaded with the new value.
The validation button is removed.
