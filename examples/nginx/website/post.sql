-- Display the post content using the card component
SELECT 'card' as component,
       'Post Details' as title,
       1 as columns;
SELECT p.title as title,
       u.username as subtitle,
       p.content as description,
       p.created_at as footer
FROM posts p
JOIN users u ON p.user_id = u.id
WHERE p.id = $id;

-- Add a divider
SELECT 'divider' as component;

-- Display comments using the list component
SELECT 'list' as component,
       'Comments' as title;
SELECT u.username as title,
       c.content as description,
       c.created_at as subtitle,
       'user' as icon,
       CASE 
         WHEN c.user_id = p.user_id THEN 'blue'
         ELSE 'gray'
       END as color
FROM comments c
JOIN users u ON c.user_id = u.id
JOIN posts p ON c.post_id = p.id
WHERE c.post_id = $id
ORDER BY c.created_at DESC;

-- Add a divider
SELECT 'divider' as component;

-- Add a comment form
SELECT 'form' as component,
       'Add a comment' as title,
       'Post comment' as validate,
       '/add_comment.sql?id=' || $id as action;

SELECT 'textarea' as type,
       'content' as name,
       'Your comment' as label,
       'Write your comment here' as placeholder,
       true as required;
