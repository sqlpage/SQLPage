-- Each matching parameter or example points back to its documentation page.
CREATE VIRTUAL TABLE documentation_fts USING fts5(
    title, description, example_source,
    page UNINDEXED, parameters UNINDEXED
);

INSERT INTO documentation_fts(title, description, page)
SELECT title, description, link FROM guide;

INSERT INTO documentation_fts(title, description, page, parameters)
SELECT name || ' component', description, '/component.sql', json_object('component', name)
FROM component
UNION ALL
SELECT component || ' component: ' || name, coalesce(description_md, description), '/component.sql', json_object('component', component)
FROM parameter
UNION ALL
SELECT 'sqlpage.' || name || '()', description_md, '/functions.sql', json_object('function', name)
FROM sqlpage_functions
UNION ALL
SELECT 'sqlpage.' || function || '(' || name || ')', description_md, '/functions.sql', json_object('function', function)
FROM sqlpage_function_parameters
UNION ALL
SELECT title, description, '/blog.sql', json_object('post', title)
FROM blog_posts;

INSERT INTO documentation_fts(title, description, example_source, page, parameters)
SELECT component || ' component example', description, properties, '/component.sql', json_object('component', component)
FROM example;
