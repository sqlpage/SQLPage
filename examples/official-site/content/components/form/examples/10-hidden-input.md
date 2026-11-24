---
rows:
  - component: form
    validate: Delete
    validate_color: red
  - type: hidden
    name: resource_id
    value: '1234'
  - name: confirm
    label: Please type "sensitive resource" here to confirm the deletion
    required: true
---

When you want to include some information in the form data, but not display it to the user, you can use a hidden field.

This can be used to track simple data such as the current user's id,
or to implement more complex flows, such as a multi-step form,
where the user is redirected to a different page after each step.

This can also be used to implement [CSRF protection](https://en.wikipedia.org/wiki/Cross-site_request_forgery#Synchronizer_token_pattern),
if your website has authenticated users that can perform sensitive actions through simple links.
But note that SQLPage cookies already have the `SameSite=strict` attribute by default, which protects you against CSRF attacks by default in most cases.
