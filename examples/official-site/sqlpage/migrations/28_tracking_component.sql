INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'tracking',
        'Component for visualising activity logs or other monitoring-related data.',
        'timeline-event-text',
        '0.18.0'
    );

INSERT INTO parameter (
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
VALUES (
        'tracking',
        'title',
        'Title of the tracking component.',
        'TEXT',
        TRUE,
        FALSE
    ),
    (
        'tracking',
        'information',
        'A short text displayed below the title.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'tracking',
        'description',
        'A short paragraph.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'tracking',
        'description_md',
        'A short paragraph formatted using markdown.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'tracking',
        'width',
        'Width of the component, between 1 and 12.',
        'NUMBER',
        TRUE,
        TRUE
    ),
    (
        'tracking',
        'placement',
        'Position of the tooltip (e.g. top, bottom, right, left)',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'tracking',
        'color',
        'Color of the tracked item (e.g. success, warning, danger)',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'tracking',
        'title',
        'Description of the state.',
        'TEXT',
        FALSE,
        FALSE
    );
-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES
    (
        'tracking', 
        'A basic example of servers tracking component', 
            JSON(
                '[
                {"component":"tracking","title":"Servers status"},
                {"title":"No data"},
                {"title":"No data"},
                {"title":"No data"},
                {"title":"No data"},
                {"title":"No data"},
                {"title":"No data"},
                {"title":"No data"},
                {"title":"No data"}
                ]'
            )
    ),      
    (
        'tracking', 
        'An example of servers tracking component', 
            JSON(
                '[
                {"component":"tracking","title":"Servers status","information":"60% are running","description_md":"Status of all **currently running servers**","placement":"top","width":4},
                {"color":"success","title":"operational"},
                {"color":"success","title":"operational"},
                {"color":"success","title":"operational"},
                {"color":"danger","title":"Downtime"},
                {"title":"No data"},
                {"color":"success","title":"operational"},
                {"color":"warning","title":"Big load"},
                {"color":"success","title":"operational"}
                ]'
            )
    );


