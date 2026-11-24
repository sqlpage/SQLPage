---
rows:
  - component: toast
    id: toast-stack-one
    trigger: queued notifications
    title: Import started
    description: Preparing records.
    duration: 0
  - component: toast
    id: toast-stack-two
    trigger: queued notifications
    title: Import running
    description: Processing records.
    duration: 0
  - component: toast
    id: toast-short
    trigger: queued notifications
    title: Temporary update
    description: This message closes shortly.
    duration: 2000
  - component: button
  - title: Show queued notifications
    link: '#queued notifications'
---

Several persistent queued notifications. Toasts with the same position share a container and stack instead of overlapping.
