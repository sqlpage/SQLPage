INSERT INTO comments (post_id, user_id, content) VALUES ($id, 1, :content);
SELECT 'redirect' as component, CONCAT('/post/', $id) AS link;
