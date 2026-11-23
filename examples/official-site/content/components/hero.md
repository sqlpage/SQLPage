---
icon: home
category: page-layout
summary: Introduce a page with a headline, media, and a call to action.
top:
  - name: autoplay
    type: BOOLEAN
    description: Automatically start playing the video
  - name: description
    type: TEXT
    description: A description of the page. Displayed below the title, in smaller characters and slightly greyed
      out.
  - name: description_md
    type: TEXT
    description: A description of the page. Displayed below the title, in smaller characters and slightly greyed
      out - formatted using markdown.
  - id
  - name: image
    type: URL
    description: The URL of an image to display next to the page title.
  - name: link
    type: URL
    description: Creates a large "call to action" button below the description, linking to the specified URL.
  - name: link_text
    type: TEXT
    description: The text to display in the call to action button. Defaults to "Go".
  - name: loop
    type: BOOLEAN
    description: Loop the video
  - name: muted
    type: BOOLEAN
    description: Mute the video
  - name: nocontrols
    type: BOOLEAN
    description: Hide the video controls (play, pause, volume, etc.), and autoplay the video.
  - name: poster
    type: URL
    description: URL of the image to be displayed before the video starts. Ignored if no video is present.
  - name: reverse
    type: BOOLEAN
    description: 'Reverse the order of the image and text: the image will be on the left, and the text on the
      right. On small screens, the image will be above the text.'
  - name: title
    type: TEXT
    description: The title of your page. Will be shown in very large characters at the top.
  - name: video
    type: URL
    description: The URL of a video to display next to the page title.
row:
  - name: color
    type: COLOR
    description: Color of the badge behind the feature section's icon. Defaults to "success".
  - name: description
    type: TEXT
    description: Description of the feature section.
  - name: description_md
    type: TEXT
    description: Description of the feature section - formatted using markdown.
  - name: icon
    type: ICON
    description: Icon of the feature section.
  - name: link
    type: TEXT
    description: An URL to which the user should be taken when they click on the section title.
  - name: title
    type: TEXT
    description: The name of a single feature section highlighted by this hero.
---

Display a large title and description for your page, with an optional large illustrative image. Useful in your home page, for instance.
