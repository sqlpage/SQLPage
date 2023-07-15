SELECT * FROM sqlpage_shell LIMIT 1;

SELECT 'list' as component,
    COALESCE(
        (SELECT name FROM topic WHERE id = $topic),
        'Recent blog posts'
    ) as title;

SELECT post.title as title,
    'post.sql?id=' || post.id as link,
    'Published on ' || created_at as description,
    CASE
        WHEN created_at > date('now', '-2 days') THEN 'red'
        ELSE NULL
    END as color,
    topic.icon as icon,
    created_at > date('now', '-2 days') as active
FROM post
    LEFT JOIN topic ON topic.id = post.main_topic_id
WHERE $topic IS NULL
    OR topic.id = $topic
    OR EXISTS (
        SELECT 1
        FROM topic_post
        WHERE topic_post.topic_id = $topic
            AND topic_post.post_id = post.id
    )
ORDER BY created_at DESC;

SELECT 'text' AS component;
SELECT 'No blog post yet. ' AS contents WHERE NOT EXISTS (SELECT 1 FROM post);
SELECT 'Write a post !' AS contents, 'write.sql' AS link;