---
rows:
  - component: html
    html: '<div class="d-flex gap-3 mb-4" style="height: 40px">'
  - html: '<div class="progress h-100"><div class="progress-bar progress-bar-striped progress-bar-animated"
      style="width: 20%">20%</div></div>'
  - html: '<div class="progress h-100"><div class="progress-bar progress-bar-striped progress-bar-animated  bg-danger"
      style="width: 80%">80%</div></div>'
  - html: </div>
---

Include multiple html snippets as row-level parameters. Again, be careful what you include in the HTML content. If the data comes from a user, it can be manipulated to include malicious code.
