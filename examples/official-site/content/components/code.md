---
icon: code
since: 0.19.0
category: page-layout
summary: Display source code with syntax highlighting.
top:
  - class
  - id
row:
  - name: contents
    type: TEXT
    required: true
    description: A block of code.
  - name: description
    type: TEXT
    description: Description of the snipet of code.
  - name: description_md
    type: TEXT
    description: Rich text in the markdown format. Among others, this allows you to write bold text using `**bold**`,
      italics using `*italics*`, and links using `[text](https://example.com)`.
  - name: language
    type: TEXT
    description: Set the programming language name.
  - name: title
    type: TEXT
    description: Set the heading level (default level is 1)
---

Displays one or many blocks of code from a programming language or formated text as XML or JSON.
