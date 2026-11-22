---
rows:
  - component: form
    title: User
    validate: Create new user
  - name: First name
    placeholder: John
  - name: Last name
    required: true
    description: We need your last name for legal purposes.
  - name: Resume
    type: textarea
  - name: Birth date
    type: date
    max: '2010-01-01'
    value: '1994-04-16'
  - name: Password
    type: password
    pattern: ^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d]{8,}$
    required: true
    description_md: '**Password Requirements:** Minimum **8 characters**, at least **one letter** & **one number**.
      *Tip:* Use a passphrase for better security!'
  - label: I accept the terms and conditions
    name: terms
    type: checkbox
    required: true
---

A user registration form, illustrating the use of required fields, and different input types.
