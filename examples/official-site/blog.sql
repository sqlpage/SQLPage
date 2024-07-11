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
        ifnull(external_url, json_object('post', title))
    ) AS link
FROM blog_posts
ORDER BY created_at DESC;