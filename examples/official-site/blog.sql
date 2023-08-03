select 'shell' as component,
    'SQLPage' as title,
    'database' as icon,
    '/' as link,
    'en-US' as language,
    'Official SQLPage website: write web applications in SQL !' as description,
    'blog' as menu_item,
    'documentation' as menu_item,
    19 as font_size,
    'Poppins' as font;

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
FROM blog_posts;