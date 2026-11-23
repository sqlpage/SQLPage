---
rows:
  - component: form
    title: User
    validate: Create new user
    validate_color: green
    reset: Clear
  - name: first_name
    label: First name
    placeholder: John
    width: 4
  - name: middle_name
    label: Middle name
    placeholder: Fitzgerald
    width: 4
  - name: last_name
    label: Last name
    placeholder: Doe
    width: 4
  - name: email
    label: Email
    placeholder: john.doe@gmail.com
    width: 12
  - name: password
    label: Password
    type: password
    width: 6
  - name: password_confirmation
    label: Password confirmation
    type: password
    width: 6
  - name: terms
    label: I accept the terms and conditions
    type: checkbox
    required: true
---

This example illustrates the use of custom validation buttons and half-width fields.
