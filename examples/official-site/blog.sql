select 'redirect' as component, '/blog.sql' as link
where ($post IS NULL AND sqlpage.path() <> '/blog.sql') OR ($post IS NOT NULL AND NOT EXISTS (SELECT 1 FROM blog_posts WHERE title = $post));

select 'shell-docs' as component, coalesce($post || ' - ', '') || 'SQLPage Blog' as title;

SELECT 'text' AS component,
        true as article,
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