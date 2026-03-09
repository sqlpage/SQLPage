SELECT 'list' AS component,
       'Todo List' AS title;

SELECT title,
       CASE WHEN done THEN 'complete' ELSE 'pending' END AS description,
       'add_todo.sql?id=' || id || '&done=' || (NOT done)::text AS link
FROM todos
ORDER BY created_at DESC;

SELECT 'form' AS component, 'Add a todo' AS title, 'add_todo.sql' AS action;
SELECT 'title' AS name, 'What do you need to do?' AS placeholder;
