INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('badge', 'label', '
A component for displaying one or multiple badges, with several options for customizing their appearance. 
The badges are arranged horizontally. They are useful for representing a classification of data. A badge can contain a link to navigate through the informations.
', '0.47.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'badge', * FROM (VALUES
    -- Top-level parameters
    ('center','If true, the component is centered in the page. Default is false.','BOOLEAN',TRUE,TRUE),
    ('size', 'The size of the badge (e.g., sm, lg).','TEXT',TRUE,TRUE),
    ('light', 'If true, creates a light version of the badge. Default is false.','BOOLEAN',TRUE,TRUE),
    ('pill', 'If true, creates a badge with rounded corners. Default is false.','BOOLEAN',TRUE,TRUE),
    ('outline', 'If true, creates a badge with a border and no fill. Default is false.','BOOLEAN',TRUE,TRUE),
    -- Item-level parameters
    ('title','The text displayed on the badge.','TEXT',FALSE,FALSE),
    ('color','The color of the badge (e.g., red, green, blue, but also primary, warning, danger, etc.). Only base color names are supported.','TEXT',FALSE,TRUE),
    ('link','Add a link to the badge and make it clickable.','URL',FALSE,TRUE)
) x;

-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES (
        'badge',
        'A group of basic badges.',
        JSON(
            '[
                {
                    "component": "badge"
                },
                {
                    "title": "Fantasy",
                },
                {
                    "title": "Horror",
                },
                {
                    "title": "Science fiction"
                }
            ]'
        )),
        (
        'badge',
        'A group of badges with a border around each.',
        JSON(
            '[
                {
                    "component": "badge",
                    "outline": true
                },
                {
                    "title": "Fantasy"
                },
                {
                    "title": "Horror"
                },
                {
                    "title": "Science fiction"
                }
            ]'
        )),
        (
        'badge',
        'A group of colored badges with rounded corners. Each badge contains a link.',
        JSON(
            '[
                {
                    "component": "badge",
                    "pill": true
                },
                {
                    "title": "Fantasy",
                    "color": "orange",
                    "link": "component.sql?component=badge"
                },
                {
                    "title": "Horror",
                    "color": "red",
                    "link": "component.sql?component=badge"
                },
                {
                    "title": "Science fiction",
                    "color": "blue",
                    "link": "#component.sql?component=badge"
                }
            ]'
        ));
    