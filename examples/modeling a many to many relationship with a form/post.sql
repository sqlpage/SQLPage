SELECT *
FROM sqlpage_shell
LIMIT 1;

SELECT 'text' AS component;

SELECT content || '

---

Published on: _' || created_at || '_ in category [' || topic.name || '](/?topic=' || topic.id || ')' || '.
    
Other associated categories: ' || (
        SELECT group_concat('[' || name || '](/?topic=' || topic_id || ')', ', ')
        FROM topic_post
            INNER JOIN topic ON topic.id = topic_post.topic_id
        WHERE post_id = $id
    ) || '.' AS contents_md
FROM post
    INNER JOIN topic ON topic.id = post.main_topic_id
WHERE post.id = $id;