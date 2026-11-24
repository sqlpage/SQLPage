---
icon: rss
since: 0.20.0
category: http-responses
summary: Publish an RSS feed from query results.
top:
  - name: author
    type: TEXT
    description: Defines the group, person, or people responsible for creating the channel.
  - name: category
    type: TEXT
    description: Defines the category of the channel. The value should be a string representing the category
      (e.g., "News", "Technology", etc.).
  - name: complete
    type: BOOLEAN
    description: Specifies that a channel is complete and will not post any more items in the future.
  - name: copyright
    type: TEXT
    description: Provides the copyright details for the channel.
  - name: description
    type: TEXT
    required: true
    description: Describes the channel.
  - name: explicit
    type: BOOLEAN
    description: Indicates whether the channel contains explicit content. The value can be either TRUE or FALSE.
  - name: funding_text
    type: TEXT
    description: The text to display with the donation/funding link. Ignored unless "funding_url" is also set.
  - name: funding_url
    type: URL
    description: Specifies the donation/funding links for the channel. The content of the tag is the recommended
      string to be used with the link.
  - name: guid
    type: TEXT
    description: The globally unique identifier (GUID) for a channel. The value is a UUIDv5.
  - name: image_url
    type: URL
    description: Provides a URL linking to the artwork for the channel.
  - name: language
    type: TEXT
    description: Defines the language of the channel, specified in the ISO 639 format. For example, "en" for
      English, "fr" for French.
  - name: link
    type: URL
    required: true
    description: Defines the hyperlink to the channel.
  - name: locked
    type: BOOLEAN
    description: Tells podcast hosting platforms whether they are allowed to import this feed.
  - name: self_link
    type: URL
    description: URL of the RSS feed.
  - name: sub_category
    type: TEXT
    description: Defines a sub-category, nested inside the "category". Ignored unless "category" is also set.
  - name: title
    type: TEXT
    required: true
    description: Defines the title of the channel.
  - name: type
    type: TEXT
    description: Specifies the channel as either episodic or serial. The value can be either "episodic" or
      "serial".
row:
  - name: block
    type: BOOLEAN
    description: Prevents a specific item from appearing in podcast listening applications.
  - name: date
    type: TEXT
    description: Indicates when the item was published (RFC-822 date-time).
  - name: description
    type: TEXT
    required: true
    description: Describes the item
  - name: duration
    type: INTEGER
    description: The duration of an item in seconds.
  - name: enclosure_length
    type: INTEGER
    description: The length in bytes of the audio/video episode content.
  - name: enclosure_type
    type: TEXT
    description: The MIME media type of the audio/video episode content (e.g., "audio/mpeg", "audio/m4a", "video/m4v",
      "video/mp4").
  - name: enclosure_url
    type: URL
    description: For podcast episodes, provides a URL linking to the audio/video episode content, in mp3, m4a,
      m4v, or mp4 format.
  - name: episode
    type: INTEGER
    description: The chronological number that is associated with an item.
  - name: episode_type
    type: TEXT
    description: Defines the type of content for a specific item. The value can be either "full", "trailer",
      or "bonus".
  - name: explicit
    type: BOOLEAN
    description: Indicates whether the item contains explicit content. The value can be either TRUE or FALSE.
  - name: guid
    type: TEXT
    description: The globally unique identifier (GUID) for an item.
  - name: image_url
    type: URL
    description: Provides a URL linking to the artwork for the item.
  - name: link
    type: URL
    required: true
    description: Defines the hyperlink to the item (blog post URL, etc.).
  - name: season
    type: INTEGER
    description: The chronological number associated with an item's season.
  - name: title
    type: TEXT
    required: true
    description: Defines the title of the feed item (episode name, blog post title, etc.).
  - name: transcript_type
    type: TEXT
    description: The type of the transcript or closed captions file for the item (e.g., "text/plain", "text/html",
      "text/vtt", "application/json", "application/x-subrip").
  - name: transcript_url
    type: URL
    description: A link to a transcript or closed captions file for the item.
---

Produces a data flow in the RSS format.
Can be used to generate a podcast feed.
To use this component, you must first return an HTTP header with the "application/rss+xml" content type (see http_header component). Next, you must use the shell-empty component to avoid that SQLPage generates HTML code.
