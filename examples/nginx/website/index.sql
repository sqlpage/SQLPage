SELECT 'list' AS component, 'Blog Posts' AS title;

SELECT 
    p.title,
    u.username AS description,
    'user' AS icon,
    '/post/' || p.id AS link
FROM posts p
JOIN users u ON p.user_id = u.id
ORDER BY p.created_at DESC;