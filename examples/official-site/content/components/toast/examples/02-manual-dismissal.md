---
rows:
  - component: toast
    id: toast-dismissible
    trigger: persistent-error
    title: Could not save
    description: Review the highlighted fields and try again.
    icon: alert-triangle
    color: black
    duration: 0
    dismissible: true
  - component: toast
    id: toast-nondismissible
    trigger: persistent-status
    title: Connection unavailable
    description: This persistent notification has no manual close control.
    duration: 0
    dismissible: false
  - component: button
  - title: Show dismissible error
    link: '#persistent-error'
    color: red
  - title: Show non-dismissible status
    link: '#persistent-status'
---

A persistent error notification that disables automatic dismissal and requires the user to activate its close button.
