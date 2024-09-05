INSERT INTO component(name, icon, description) VALUES
    ('foldable', 'chevrons-down', 'A foldable list of elements which can be expanded individually.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'foldable', * FROM (VALUES
    ('id', 'ID attribute added to the container in HTML. Used for targeting through CSS or for scrolling via links.', 'TEXT', TRUE, TRUE),
    ('title', 'Title of the foldable item, displayed on the button.', 'TEXT', FALSE, TRUE),
    ('description', 'Plain text description of the item, displayed when expanded.', 'TEXT', FALSE, TRUE),
    ('description_md', 'Markdown description of the item, displayed when expanded.', 'TEXT', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('foldable', 'A SQLPage-themed foldable list with Markdown', json('[
        {"component":"foldable"},
        {"title":"Quick Prototyping", "description_md": "Build a functional web app prototype in minutes using just SQL queries:\n\n- Rapid development\n- Ideal for MVPs\n- Great for internal tools\n\nLearn more about [quick prototyping](/your-first-sql-website/)."},
        {"title":"Data Visualization", "description_md": "Transform data into insights:\n\n1. **Charts**: Line, bar, pie\n2. **Graphs**: Network diagrams\n3. **Maps**: Geospatial data\n\n```sql\nSELECT ''chart'' as component;\nSELECT date as x, revenue as y FROM sales;\n```"},
        {"title":"Form Handling", "description_md": "Effortless form processing:\n\n- *User input collection*\n- *Data validation*\n- *Database updates*\n\n> SQLPage handles the entire form lifecycle, from rendering to processing."}
    ]'));