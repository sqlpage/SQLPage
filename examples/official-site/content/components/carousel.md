---
icon: carousel-horizontal
since: 0.18.3
category: display-data
summary: Show a slideshow of images with optional captions and controls.
top:
  - name: auto
    type: BOOLEAN
    description: Whether to automatically cycle through the carousel items. Default is false.
  - name: center
    type: BOOLEAN
    description: Whether to center the carousel.
  - class
  - name: controls
    type: BOOLEAN
    description: Whether to show the control links to go previous or next item.
  - name: delay
    type: INTEGER
    description: Specify the delay, in milliseconds, between two images.
  - name: fade
    type: BOOLEAN
    description: Whether to apply the fading effect.
  - id
  - name: indicators
    type: TEXT
    description: Style of image indicators (square or dot).
  - name: title
    type: TEXT
    description: A name to display at the top of the carousel.
  - name: vertical
    type: BOOLEAN
    description: Whether to use the vertical image indicators.
  - name: width
    type: INTEGER
    description: Width of the component, between 1 and 12. Default is 12.
row:
  - name: description
    type: TEXT
    description: A short paragraph.
  - name: description_md
    type: TEXT
    description: A short paragraph formatted using markdown.
  - name: height
    type: INTEGER
    description: The height of the image, in pixels.
  - name: image
    type: URL
    required: true
    description: The URL (absolute or relative) of an image to display in the carousel.
  - name: title
    type: TEXT
    description: Add caption to the slide.
  - name: width
    type: INTEGER
    description: The width of the image, in pixels.
---

A carousel is used to display images. When used with multiple images, it will cycle through them automatically or with controls, creating a slideshow.
