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
    CASE 
        WHEN external_url IS NOT NULL
        THEN external_url
    ELSE 
        '?post=' || title
    END AS link
FROM blog_posts
ORDER BY created_at DESC;