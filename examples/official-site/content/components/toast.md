---
icon: notification
since: 0.46.0
category: forms-and-feedback
summary: Show a brief notification above the page.
top:
  - class
  - name: color
    type: COLOR
    description: Optional Tabler color. The default is the neutral toast appearance; colored variants use the
      matching contrasting foreground utility.
  - name: description
    type: TEXT
    description: Escaped plain-text body. This is used only when `description_md` is not supplied.
  - name: description_md
    type: TEXT
    description: Rich-text alternative to `description`, rendered as Markdown. When both properties are supplied,
      `description_md` takes precedence.
  - name: dismissible
    type: BOOLEAN
    description: Whether to render an accessible manual close button. Defaults to true and is independent of
      automatic dismissal.
  - name: duration
    type: INTEGER
    description: Automatic dismissal delay in milliseconds. Defaults to 5000. Set to 0 to keep the toast visible
      until manually dismissed (when `dismissible` is true) or the page is left.
  - name: icon
    type: ICON
    description: Optional [Tabler icon](https://tabler.io/icons) name.
  - id
  - name: position
    type: TEXT
    description: 'Screen placement: `top-start`, `top-center`, `top-end`, `bottom-start`, `bottom-center`,
      or `bottom-end`. Defaults to `top-end`; invalid values safely fall back to that default.'
  - name: title
    type: TEXT
    description: Optional notification heading.
  - name: trigger
    type: TEXT
    description: Optional URL fragment that opens the toast without reloading the page, with or without the
      leading `#`. When set, the toast does not open on page load and can be opened repeatedly by a link or
      button whose target is that fragment. Multiple toasts can share a trigger to open as a stack.
---

Displays a brief notification above the page. Each top-level `toast` row creates one notification, and consecutive toasts at the same position are queued in a shared stack.

Ordinary notifications use `role="status"` and `aria-live="polite"`. Colored variants retain a readable contrasting foreground. Automatic dismissal and the manual close control are configured independently.
