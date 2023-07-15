INSERT INTO post (title, content, main_topic_id)
VALUES (:Title, :Content, :"Main Topic");

INSERT INTO topic_post (topic_id, post_id)
SELECT CAST(topic.value AS INTEGER),
    last_insert_rowid()
FROM json_each(:Topics) AS topic -- we receive the value from checkboxes as a JSON array
WHERE topic.value IS NOT NULL;

SELECT 'redirect' AS component, 'post.sql?id=' || last_insert_rowid() AS link;