INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('status_bar', 'layout-bottombar',
     'Display short status messages and links in a single horizontal bar. Each row adds one segment. When there are several segments, the last one aligns to the far edge. If the segments do not all fit, scroll the bar horizontally to see the rest.',
     '0.49.0');

INSERT INTO parameter(component, name, description, type, top_level, optional)
SELECT 'status_bar', * FROM (VALUES
    ('fixed', 'Keep the bar at the bottom of the screen while the page scrolls. Defaults to true. A fixed bar overlays page content, so leave space below your last item if needed. Set to false to place the bar in the page where the SQL appears.', 'BOOLEAN', TRUE, TRUE),
    ('label', 'Accessible name for the bar. Defaults to "Status bar".', 'TEXT', TRUE, TRUE),
    ('title', 'Text displayed in the segment.', 'TEXT', FALSE, TRUE),
    ('icon', 'Icon displayed before the text.', 'ICON', FALSE, TRUE),
    ('color', 'Color of the segment text and icon.', 'COLOR', FALSE, TRUE),
    ('tooltip', 'Text shown when hovering over the segment or focusing its link. For an icon without a title, supply a tooltip to give the segment an accessible name.', 'TEXT', FALSE, TRUE),
    ('link', 'URL to open when the segment is clicked.', 'URL', FALSE, TRUE)
) p;

INSERT INTO example(component, description, properties) VALUES
    ('status_bar', 'A query workspace shows the database connection, row count, and a link to the function reference. Omit `fixed` to pin this bar to the bottom of the screen.', json('[
        {"component":"status_bar","label":"Query status","fixed":false},
        {"title":"Connected","icon":"database","color":"green","tooltip":"Connected to the reporting database"},
        {"title":"24 rows"},
        {"title":"SQL functions","icon":"book","link":"/functions.sql"}
    ]'));
