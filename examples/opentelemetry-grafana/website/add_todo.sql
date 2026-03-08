-- Toggle an existing todo
UPDATE todos SET done = $done::boolean WHERE id = $id::int;

-- Insert a new todo if title is provided via the form (POST)
INSERT INTO todos (title)
SELECT :title WHERE :title IS NOT NULL AND length(:title) > 0;

SELECT 'redirect' AS component, '/' AS link;
