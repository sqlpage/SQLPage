INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('foldable', 'chevrons-down', 'A foldable list of elements which can be expanded individually.', '0.29.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'foldable', * FROM (VALUES
    ('id', 'ID attribute added to the container in HTML. Used for targeting through CSS or for scrolling via links. When set at the top level, applies to the entire foldable component.', 'TEXT', TRUE, TRUE),
    ('class', 'CSS class(es) to add to the foldable container. When set at the top level, applies to the entire foldable component.', 'TEXT', TRUE, TRUE),
    ('id', 'ID attribute added to individual foldable items. Used for targeting through CSS or for scrolling via links.', 'TEXT', FALSE, TRUE),
    ('class', 'CSS class(es) to add to individual foldable items.', 'TEXT', FALSE, TRUE),
    ('title', 'Title of the foldable item, displayed on the button.', 'TEXT', FALSE, TRUE),
    ('description', 'Plain text description of the item, displayed when expanded.', 'TEXT', FALSE, TRUE),
    ('description_md', 'Markdown description of the item, displayed when expanded.', 'TEXT', FALSE, TRUE),
    ('expanded', 'If set to TRUE, the foldable item starts in an expanded state. Defaults FALSE', 'BOOLEAN', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('foldable', 'A single foldable paragraph of text', json('[
        {"component":"foldable"},
        {"title":"The foldable component", "description": "This is a simple foldable component. It can be used to show and hide content. It is a list of items, each with a title and a description. The description is displayed when the item is expanded."},
    ]'));

INSERT INTO example(component, description, properties) VALUES
    ('foldable', 'A SQLPage-themed foldable list with Markdown', json('[
        {"component":"foldable"},
        {"title":"Quick Prototyping", "description_md": "Build a functional web app prototype in minutes using just SQL queries:\n\n- Rapid development\n- Ideal for MVPs\n- Great for internal tools\n\nLearn more about [quick prototyping](/your-first-sql-website/).", "expanded": true},
        {"title":"Data Visualization", "description_md": "Quickly transform your database into useful insights:\n\n1. **Charts**: Line, bar, pie\n2. **KPIs**: Appealing visualizations of key metrics\n3. **Maps**: Geospatial data\n\nAs simple as:\n\n```sql\nSELECT ''chart'' as component;\nSELECT date as x, revenue as y FROM sales;\n```"},
        {"title":"Don''t stare, interact!", "description_md": "SQLPage is not just a passive *Business Intelligence* tool. With SQLPage, you can act upon user input:\n\n- *User input collection*: Building a form is just as easy as building a chart.\n- *Data validation*: Write your own validation rules in SQL.\n- *Database updates*: `INSERT` and `UPDATE` are first-class citizens.\n- *File uploads*: Upload `CSV` and other files, store and display them the way you want.\n\n> Let users interact with your data, not just look at it!"}
    ]'));