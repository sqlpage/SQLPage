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
        'placement',
        'Position of the tooltip (e.g. top, bottom, right, left)',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'tracking',
        'state',
        'State of the tracked item (e.g. success, warning, danger)',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'tracking',
        'tooltip',
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
        'An example of servers tracking component', 
            JSON(
                '[
                {"component":"tracking","title":"Servers status","information":"60% are running","description_md":"Status of all **currently running servers**","placement":"top"},
                {"state":"success","tooltip":"operational"},
                {"state":"success","tooltip":"operational"},
                {"state":"success","tooltip":"operational"},
                {"state":"danger","tooltip":"Downtime"},
                {"tooltip":"No data"},
                {"state":"success","tooltip":"operational"},
                {"state":"warning","tooltip":"Big load"},
                {"state":"success","tooltip":"operational"}
                ]'
            )
    );


