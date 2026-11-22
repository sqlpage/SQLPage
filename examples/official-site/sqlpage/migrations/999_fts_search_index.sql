CREATE VIRTUAL TABLE documentation_fts USING fts5(
    title, description, example_source,
    page UNINDEXED, parameters UNINDEXED, fragment UNINDEXED
);

INSERT INTO documentation_fts(title, description, page)
SELECT title, description, link FROM guide
UNION ALL
SELECT CASE kind WHEN 'component' THEN name || ' component' ELSE 'sqlpage.' || name || '()' END,
    description_md, link FROM reference_entry;

INSERT INTO documentation_fts(title, description, page, fragment)
SELECT CASE p.kind WHEN 'component' THEN p.entry || ' component: ' || p.name
        ELSE 'sqlpage.' || p.entry || '(' || p.name || ')' END,
    p.description_md, e.link, p.anchor
FROM reference_parameter AS p
JOIN reference_entry AS e ON e.kind = p.kind AND e.name = p.entry;

INSERT INTO documentation_fts(title, description, example_source, page, fragment)
SELECT component || ' component example', description, properties, e.link, 'example-' || example_number
FROM example_with_source AS x
JOIN reference_entry AS e ON e.kind = 'component' AND e.name = x.component;

INSERT INTO documentation_fts(title, description, page, parameters)
SELECT title, description, '/blog.sql', json_object('post', title) FROM blog_posts;
