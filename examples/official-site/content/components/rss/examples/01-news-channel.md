### An RSS channel about SQLPage latest news.

```sql
select 'http_header' as component, 'application/rss+xml' as content_type;
select 'shell-empty' as component;
select
  'rss' as component,
  'SQLPage blog' as title,
  'https://sql-page.com/blog.sql' as link,
  'latest news about SQLpage' as description,
  'en' as language,
  'Technology' as category,
  FALSE as explicit,
  'https://sql-page.com/favicon.ico' as image_url,
  'Ophir Lojkine' as author,
  'https://github.com/sponsors/lovasoa' as funding_url,
  'episodic' as type;
select
  'Hello everyone !' as title,
  'https://sql-page.com/blog.sql?post=Come%20see%20me%20build%20twitter%20live%20on%20stage%20in%20Prague' as link,
  'If some of you european SQLPagers are around Prague this december, I will be giving a talk about SQLPage at pgconf.eu on December 14th.' as description,
  'http://127.0.0.1:8080/sqlpage_introduction_video.mp4' as enclosure_url,
  123456789 as enclosure_length,
  'video/mp4' as enclosure_type,
  '2023-12-04' as date;
```

Once you have your rss feed ready, you can submit it to podcast directories like
[Apple Podcasts](https://podcastsconnect.apple.com/my-podcasts),
[Spotify](https://podcasters.spotify.com/),
[Google Podcasts](https://podcastsmanager.google.com/)...
