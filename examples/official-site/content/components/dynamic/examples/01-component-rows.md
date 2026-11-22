---
rows:
  - component: dynamic
    properties: '[{"component":"text"}, {"contents":"Hello, I am a dynamic component !", "italics":true}]'
---

The dynamic component has a single top-level property named `properties`, but it can render any number of other components.
Let's start with something simple to illustrate the logic. We'll render a `text` component with two row-level properties: `contents` and `italics`.
