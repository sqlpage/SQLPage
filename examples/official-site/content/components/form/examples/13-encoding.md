---
rows:
  - component: form
    method: post
    enctype: multipart/form-data
    title: Submit with different encoding types
    validate: Submit with form encoding type
    action: examples/handle_enctype.sql
  - name: data
    type: text
    label: Data
    required: true
  - name: percent_encoded
    type: submit
    label: Submit as
    width: 4
    formaction: examples/handle_enctype.sql
    formenctype: application/x-www-form-urlencoded
    value: application/x-www-form-urlencoded
  - name: multipart_form_data
    type: submit
    label: Submit as
    width: 4
    formaction: examples/handle_enctype.sql
    formenctype: multipart/form-data
    value: multipart/form-data
---

## Form Encoding

You can specify the way form data should be encoded by setting the `enctype`
top-level property on the form.

You may also specify `formenctype` on `submit` and `image` type inputs.
This will take precedence over the `enctype` specified on the form and is
useful in the case there are multiple `submit` buttons on the form.
For example, an external site may have specific requirements on encoding type.

As a rule of thumb, ``multipart/form-data`` is best when fields may contain
copious non-ascii characters or for binary data such as an image or a file.
However, ``application/x-www-form-urlencoded`` creates less overhead when
many short ascii text values are submitted.
