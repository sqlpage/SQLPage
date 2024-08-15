SELECT * FROM sqlpage_shell LIMIT 1;

SELECT 'form' AS component, 'Publish !' AS validate, 'write_submit.sql' AS action;
SELECT 'Title' AS name, 'text' AS type, 'Title of the post. Write something that makes people want to click !' AS description, TRUE AS required;
SELECT 'Content' AS name, 'textarea' AS type, 'The content of the post. Write something exciting !' AS description, TRUE AS required;
SELECT 'Main Topic' AS name,
    'select' AS type,
    'The main topic of the post. This will be used to display the post in the main page.' AS description,
    json_group_array(json_object('label', name, 'value', id)) AS options
FROM topic;
SELECT 'Topics[]' AS name, 'checkbox' AS type, 'Check if this post should also appear in the "' || topic.name || '" category.' AS description, topic.id AS value, topic.name AS label FROM topic;