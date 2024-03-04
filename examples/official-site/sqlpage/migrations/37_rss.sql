-- Documentation for the RSS component
INSERT INTO component (name, description, icon, introduced_in_version) VALUES (
    'rss',
    'Produces a data flow in the RSS format.
Can be used to generate a podcast feed.
To use this component, you must first return an HTTP header with the "application/rss+xml" content type (see http_header component). Next, you must use the shell-empty component to avoid that SQLPage generates HTML code.',
    'rss',
    '0.20.0'
);

INSERT INTO parameter (component,name,description,type,top_level,optional) VALUES (
    'rss',
    'title',
    'Defines the title of the channel.',
    'TEXT',
    TRUE,
    FALSE
),(
    'rss',
    'link',
    'Defines the hyperlink to the channel.',
    'URL',
    TRUE,
    FALSE
),(
    'rss',
    'description',
    'Describes the channel.',
    'TEXT',
    TRUE,
    FALSE
),(
    'rss',
    'language',
    'Defines the language of the channel, specified in the ISO 639 format. For example, "en" for English, "fr" for French.',
    'TEXT',
    TRUE,
    TRUE
),(
    'rss',
    'category',
    'Defines the category of the channel. The value should be a string representing the category (e.g., "News", "Technology", etc.).',
    'TEXT',
    TRUE,
    TRUE
),(
    'rss',
    'explicit',
    'Indicates whether the channel contains explicit content. The value can be either TRUE or FALSE.',
    'BOOLEAN',
    TRUE,
    TRUE
),(
    'rss',
    'image_url',
    'Provides a URL linking to the artwork for the channel.',
    'URL',
    TRUE,
    TRUE
),(
    'rss',
    'author',
    'Defines the group, person, or people responsible for creating the channel.',
    'TEXT',
    TRUE,
    TRUE
),(
    'rss',
    'copyright',
    'Provides the copyright details for the channel.',
    'TEXT',
    TRUE,
    TRUE
),(
    'rss',
    'funding_url',
    'Specifies the donation/funding links for the channel. The content of the tag is the recommended string to be used with the link.',
    'URL',
    TRUE,
    TRUE
),(
    'rss',
    'type',
    'Specifies the channel as either episodic or serial. The value can be either "episodic" or "serial".',
    'TEXT',
    TRUE,
    TRUE
),(
    'rss',
    'complete',
    'Specifies that a channel is complete and will not post any more items in the future.',
    'BOOLEAN',
    TRUE,
    TRUE
),(
    'rss',
    'locked',
    'Tells podcast hosting platforms whether they are allowed to import this feed.',
    'BOOLEAN',
    TRUE,
    TRUE
),(
    'rss',
    'guid',
    'The globally unique identifier (GUID) for a channel. The value is a UUIDv5.',
    'TEXT',
    TRUE,
    TRUE
),(
    'rss',
    'title',
    'Defines the title of the feed item (episode name, blog post title, etc.).',
    'TEXT',
    FALSE,
    FALSE
),(
    'rss',
    'link',
    'Defines the hyperlink to the item (blog post URL, etc.).',
    'URL',
    FALSE,
    FALSE
),(
    'rss',
    'description',
    'Describes the item',
    'TEXT',
    FALSE,
    FALSE
),(
    'rss',
    'date',
    'Indicates when the item was published (RFC-822 date-time).',
    'TEXT',
    FALSE,
    TRUE
),(
    'rss',
    'enclosure_url',
    'For podcast episodes, provides a URL linking to the audio/video episode content, in mp3, m4a, m4v, or mp4 format.',
    'URL',
    FALSE,
    TRUE
),(
    'rss',
    'enclosure_length',
    'The length in bytes of the audio/video episode content.',
    'INTEGER',
    FALSE,
    TRUE
),(
    'rss',
    'enclosure_type',
    'The MIME media type of the audio/video episode content (e.g., "audio/mpeg", "audio/m4a", "video/m4v", "video/mp4").',
    'TEXT',
    FALSE,
    TRUE
),(
    'rss',
    'guid',
    'The globally unique identifier (GUID) for an item.',
    'TEXT',
    FALSE,
    TRUE
),(
    'rss',
    'episode',
    'The chronological number that is associated with an item.',
    'INTEGER',
    FALSE,
    TRUE
),(
    'rss',
    'season',
    'The chronological number associated with an item''s season.',
    'INTEGER',
    FALSE,
    TRUE
),(
    'rss',
    'episode_type',
    'Defines the type of content for a specific item. The value can be either "full", "trailer", or "bonus".',
    'TEXT',
    FALSE,
    TRUE
),(
    'rss',
    'block',
    'Prevents a specific item from appearing in podcast listening applications.',
    'BOOLEAN',
    FALSE,
    TRUE
),(
    'rss',
    'explicit',
    'Indicates whether the item contains explicit content. The value can be either TRUE or FALSE.',
    'BOOLEAN',
    FALSE,
    TRUE
),(
    'rss',
    'image_url',
    'Provides a URL linking to the artwork for the item.',
    'URL',
    FALSE,
    TRUE
),(
    'rss',
    'duration',
    'The duration of an item in seconds.',
    'INTEGER',
    FALSE,
    TRUE
),(
    'rss',
    'transcript_url',
    'A link to a transcript or closed captions file for the item.',
    'URL',
    FALSE,
    TRUE
),(
    'rss',
    'transcript_type',
    'The type of the transcript or closed captions file for the item (e.g., "text/plain", "text/html", "text/vtt", "application/json", "application/x-subrip").',
    'TEXT',
    FALSE,
    TRUE
);

-- Insert example(s) for the component
INSERT INTO example (component, description)
VALUES (
        'rss',
        '
### An RSS channel about SQLPage latest news.

```sql
select ''http_header'' as component, ''application/rss+xml'' as content_type;
select ''shell-empty'' as component;
select
  ''rss'' as component,
  ''SQLPage blog'' as title,
  ''https://sql.ophir.dev/blog.sql'' as link,
  ''latest news about SQLpage'' as description,
  ''en'' as language,
  ''Technology'' as category,
  FALSE as explicit,
  ''https://sql.ophir.dev/favicon.ico'' as image_url,
  ''Ophir Lojkine'' as author,
  ''https://github.com/sponsors/lovasoa'' as funding_url,
  ''episodic'' as type;
select
  ''Hello everyone !'' as title,
  ''https://sql.ophir.dev/blog.sql?post=Come%20see%20me%20build%20twitter%20live%20on%20stage%20in%20Prague'' as link,
  ''If some of you european SQLPagers are around Prague this december, I will be giving a talk about SQLPage at pgconf.eu on December 14th.'' as description,
  ''http://127.0.0.1:8080/sqlpage_introduction_video.webm'' as enclosure_url,
  123456789 as enclosure_length,
  ''video/webm'' as enclosure_type,
  ''2023-12-04'' as date;
```

Once you have your rss feed ready, you can submit it to podcast directories like
[Apple Podcasts](https://podcastsconnect.apple.com/my-podcasts),
[Spotify](https://podcasters.spotify.com/),
[Google Podcasts](https://podcastsmanager.google.com/)...
');