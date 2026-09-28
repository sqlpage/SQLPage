INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('lazy', 'loader', 'Load parts of a page independently while showing a placeholder. Use this for slow charts or tables so the rest of the page can appear first.', '0.47.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'lazy', * FROM (VALUES
    ('class', 'CSS classes for the outer container. Replaces the default my-2 class.', 'TEXT', TRUE, TRUE),
    ('style', 'Inline CSS for the outer container.', 'TEXT', TRUE, TRUE),
    ('embed', 'URL of the SQLPage page to load. Its contents replace the loading placeholder; the shell is omitted automatically.', 'URL', FALSE, FALSE),
    ('class', 'CSS classes for the loading placeholder. Replaces the default card my-2 classes.', 'TEXT', FALSE, TRUE),
    ('style', 'Inline CSS for the loading placeholder. Set a height to reserve space while content loads.', 'TEXT', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('lazy', 'Load a chart independently and reserve space for it while it loads.', json('[
        {"component":"lazy"},
        {"embed":"/examples/chart.sql", "style":"height:340px"}
    ]'));
