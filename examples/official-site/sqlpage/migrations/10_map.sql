INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'map',
        'Displays a map with markers on it. Useful in combination with PostgreSQL''s PostGIS or SQLite''s spatialite.',
        'map',
        '0.8.0'
    );
-- Insert the parameters for the http_header component into the parameter table
INSERT INTO parameter (
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
VALUES (
        'map',
        'latitude',
        'Latitude of the center of the map.',
        'REAL',
        TRUE,
        TRUE
    ),
    (
        'map',
        'longitude',
        'Longitude of the center of the map.',
        'REAL',
        TRUE,
        TRUE
    ),
    (
        'map',
        'zoom',
        'Zoom Level to apply to the map.',
        'REAL',
        TRUE,
        TRUE
    ),
    (
        'map',
        'latitude',
        'Latitude of the marker',
        'REAL',
        FALSE,
        FALSE
    ),
    (
        'map',
        'longitude',
        'Longitude of the marker',
        'REAL',
        FALSE,
        FALSE
    ),
    (
        'map',
        'title',
        'Title of the marker',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'map',
        'link',
        'A link to associate to the marker''s title',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'map',
        'description',
        'Plain text description of the marker, as plain text',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'map',
        'description_md',
        'Description of the marker, in markdown',
        'TEXT',
        FALSE,
        TRUE
    );
-- Insert an example usage of the map component into the example table
INSERT INTO example (component, description, properties)
VALUES (
        'map',
        'Map of Paris',
        JSON('[
            { "component": "map", "title": "Paris", "zoom": 11, "latitude": 48.85, "longitude": 2.34, "height": 400 },
            { "title": "Notre Dame", "latitude": 48.8530, "longitude": 2.3498, "description_md": "A beautiful cathedral.", "link": "https://en.wikipedia.org/wiki/Notre-Dame_de_Paris" },
            { "title": "Eiffel Tower", "latitude": 48.8584, "longitude": 2.2945, "description_md": "A tall tower. [Wikipedia](https://en.wikipedia.org/wiki/Eiffel_Tower)" }
        ]')
    );