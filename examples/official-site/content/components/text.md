---
icon: align-left
category: page-layout
summary: Write paragraphs using plain text or Markdown.
top:
  - name: article
    type: BOOLEAN
    description: Makes long texts more readable by increasing the line height, adding margins, using a serif
      font, and decorating the initial letter.
  - name: center
    type: BOOLEAN
    description: Whether to center the title.
  - name: contents
    type: TEXT
    description: A top-level paragraph of text to display, without any formatting, without having to make additional
      queries.
  - name: contents_md
    type: TEXT
    description: Rich text in the markdown format. Among others, this allows you to write bold text using `**bold**`,
      italics using `*italics*`, and links using `[text](https://example.com)`.
  - name: html
    type: TEXT
    description: Raw html code to include on the page. Don't use that if you are not sure what you are doing,
      it may have security implications.
  - id
  - name: title
    type: TEXT
    description: Text header before the paragraph.
  - name: unsafe_contents_md
    type: TEXT
    description: Markdown format with html blocks. Use this only with trusted content. See the html-blocks
      section of the Commonmark spec for additional info.
row:
  - name: bold
    type: BOOLEAN
    description: Whether the span of text should be displayed as bold.
  - name: break
    type: BOOLEAN
    description: Indicates that the current span of text starts a new paragraph.
  - name: code
    type: BOOLEAN
    description: Use a monospace font. Useful to display the text as code.
  - name: color
    type: COLOR
    description: The name of a color for this span of text.
  - name: contents
    type: TEXT
    required: true
    description: A span of text to display
  - name: contents_md
    type: TEXT
    description: Rich text in the markdown format. Among others, this allows you to write bold text using `**bold**`,
      italics using `*italics*`, and links using `[text](https://example.com)`.
  - name: italics
    type: BOOLEAN
    description: Whether the span of text should be displayed as italics.
  - name: link
    type: URL
    description: An URL to which the user should be taken when they click on this span of text.
  - name: size
    type: INTEGER
    description: A number between 1 and 6 indicating the font size.
  - name: underline
    type: BOOLEAN
    description: Whether the span of text should be underlined.
  - name: unsafe_contents_md
    type: TEXT
    description: Markdown format with html blocks. Use this only with trusted content. See the html-blocks
      section of the Commonmark spec for additional info.
---

A paragraph of text. The entire component will render as a single paragraph, with each item being rendered as a span of text inside it, the styling of which can be customized using parameters.
