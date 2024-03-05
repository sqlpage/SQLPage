select 'http_header' as component,
    'application/rss+xml' as "Content-Type";
select 'shell-empty' as component;
select 'rss' as component,
    'SQLPage blog' as title,
    'https://sql.ophir.dev/blog.sql' as link,
    'latest news about SQLpage' as description,
    'en' as language,
    'https://sql.ophir.dev/rss.sql' as self_link,
    'Technology' as category,
    '2de3f968-9928-5ec6-9653-6fc6fe382cfd' as guid;
SELECT title,
    description,
    CASE
        WHEN external_url IS NOT NULL THEN external_url
        ELSE 'https://sql.ophir.dev/blog.sql?post=' || title
    END AS link,
    created_at AS date
FROM blog_posts
ORDER BY created_at DESC;