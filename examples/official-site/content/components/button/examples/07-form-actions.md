---
rows:
  - component: form
    id: poem
    validate: ''
  - type: textarea
    name: Poem
    placeholder: Write a poem
  - component: button
  - link: ?action=save
    form: poem
    color: primary
    title: Save
  - link: ?action=preview
    form: poem
    outline: yellow
    title: Preview
---

Multiple buttons sending the same form to different pages.

We use `'' AS validate` to remove the submit button from inside the form itself,
and instead use the button component to submit the form to pages with different GET variables.

In the target page, we could then use the GET variable `$action` to determine what to do with the form data.
