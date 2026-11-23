---
icon: credit-card
category: display-data
summary: Arrange records in cards with images, links, or embedded content.
top:
  - class
  - name: columns
    type: INTEGER
    description: The number of columns in the grid of cards. This is just a hint, the grid will adjust dynamically
      to the user's screen size, rendering fewer columns if needed to fit the contents. To control the size
      of cards individually, use the `width` row-level property instead.
  - name: description
    type: TEXT
    description: A short paragraph displayed below the title.
  - name: description_md
    type: TEXT
    description: A short paragraph displayed below the title - formatted using markdown.
  - name: title
    type: TEXT
    description: Text header at the top of the list of cards.
row:
  - name: active
    type: BOOLEAN
    description: Whether this item in the grid is considered "active". Active items are displayed more prominently.
  - name: allow
    type: TEXT
    description: The permissions policy of the iframe embed code, for instance "fullscreen; clipboard-write".
      Used only when embed_mode is 'iframe'.
  - name: background_color
    type: COLOR
    description: The background color of the card.
  - class
  - name: color
    type: COLOR
    description: The name of a color, to be displayed on the left of the card to highlight it. If the embed
      parameter is enabled and you don't have a title or description, this parameter won't apply.
  - name: description
    type: TEXT
    description: |-
      The body of the card, where you put the main text contents of the card.
              This does not support rich text formatting, only plain text.
              If you want to use rich text formatting, use the `description_md` property instead.
  - name: description_md
    type: TEXT
    description: |2-

              The body of the card, in Markdown format.
              This is useful if you want to display a lot of text in the card, with many options for formatting, such as
              line breaks, **bold**, *italics*, lists, #titles, [links](target.sql), ![images](photo.jpg), etc.
  - name: embed
    type: TEXT
    description: |-
      A url whose contents will be fetched and injected into the body of this card.
              This can be used to inject arbitrary html content, but is especially useful for injecting
              the output of other sql files rendered by SQLPage. For the latter case you can pass the
              `?_sqlpage_embed` query parameter, which will skip the shell layout.
              Components inside the fetched content initialize automatically. Custom scripts that insert
              SQLPage fragments should dispatch a bubbling `fragment-loaded` event on the element containing
              the new content. Initialization includes that element and its descendants; dispatching the
              event on `document` initializes pending components throughout the page. Repeated events
              preserve existing form and widget handlers
  - name: embed_mode
    type: TEXT
    description: |-
      Set to 'iframe' to embed the target (specified through embed property) in an iframe.
              Unless this is explicitly set, the embed target is fetched and injected within the parent page. If embed_mode is set to iframe,
              You can also set height and width parameters to configure the appearance and the sandbox and allow parameters to configure
              security aspects of the iframe. Refer to the [MDN page](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe)
              for an explanation of these parameters.
  - name: footer
    type: TEXT
    description: Muted text to display at the bottom of the card.
  - name: footer_link
    type: URL
    description: An URL to which the user should be taken when they click on the footer.
  - name: footer_md
    type: TEXT
    description: Muted text to display at the bottom of the card, with rich text formatting in Markdown format.
  - name: height
    type: INTEGER
    description: Height of the iframe embed code, in pixels. Used only when embed_mode is 'iframe'.
  - name: icon
    type: ICON
    description: Name of an icon to display on the right side of the card.
  - id
  - name: link
    type: URL
    description: An URL to which the user should be taken when they click on the card.
  - name: sandbox
    type: TEXT
    description: The restrictions to apply to the iframe embed code, for instance "allow-scripts allow-same-origin".
      Used only when embed_mode is 'iframe'.
  - name: style
    type: TEXT
    description: 'Inline style property to your iframe embed code. For example "background-color: #FFFFFF"'
  - name: target
    type: TEXT
    description: 'Where the card''s link opens: "_blank" for a new tab, "_self" for the same tab, "_parent"
      for the parent frame, or "_top" for the full body of the window.'
  - name: title
    type: TEXT
    required: true
    description: Name of the card, displayed at the top.
  - name: top_image
    type: URL
    description: The URL (absolute or relative) of an image to display at the top of the card.
  - name: top_image_height
    type: INTEGER
    description: Specify the top image height, in pixels. Helps prevent layout shifts.
  - name: top_image_lazy
    type: BOOLEAN
    description: Whether the top image must be lazily loaded. Defaults to false, meaning eagerly loaded.
  - name: top_image_width
    type: INTEGER
    description: Specify the top image width, in pixels. Helps prevent layout shifts.
  - name: width
    type: INTEGER
    description: The width of the card, between 1 (smallest) and 12 (full-width). The default width is 3, resulting
      in 4 cards per line. When embed_mode is 'iframe', this is used as the width of the iframe instead, and
      defaults to 100%.
---

A grid where each element is a small card that displays a piece of data.
