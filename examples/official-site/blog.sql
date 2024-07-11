select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

SELECT 'text' AS component,
        content AS contents_md
FROM blog_posts
WHERE title = $post;

SELECT 'list' AS component,
    'SQLPage blog' AS title;
SELECT title,
    description,
    icon,
    sqlpage.link(
        COALESCE(external_url, ''),
        CASE WHEN external_url IS NULL THEN json_object('post', title) ELSE NULL END
    ) AS link
FROM blog_posts
ORDER BY created_at DESC;