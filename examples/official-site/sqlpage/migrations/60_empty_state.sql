INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('empty_state', 'info-circle', 'Empty states are placeholders for first-use, empty data, or error screens', '0.35.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'empty_state', * FROM (VALUES
    ('title','Description of the empty state.','TEXT',TRUE,FALSE),
    ('status_code','HTTP status code displayed on the top of the empty state.','INTEGER',TRUE,TRUE),
    ('icon','Name of an icon to be displayed on the top of the empty state.','ICON',TRUE,TRUE),
    ('image','The URL (absolute or relative) of an image to display at the top of the empty state.','URL',TRUE,TRUE),
    ('description','A short text displayed below the title.','TEXT',TRUE,TRUE),
    ('link_text','The text displayed on the button.','TEXT',TRUE,FALSE),
    ('link_icon','Name of an icon to be displayed on the left side of the button.','ICON',TRUE,FALSE),
    ('link','The URL to which the button should navigate when clicked.','URL',TRUE,FALSE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('empty_state', '
The empty_state component provides users with informative and visually appealing placeholders when there is no content to display in a particular section of an application or website. Its role is to enhance user experience by guiding users on what to do next, offering suggestions, or providing context about the absence of data. This component includes a title, a description, an action button and often an illustration or icon.  The empty_state component helps to reduce confusion and encourages users to take action.
',
    json('[{
        "component": "empty_state",
        "title": "No results found",
        "status_code": 404,
        "description": "Try adjusting your search or filter to find what you''re looking for.",
        "link_text": "Search again",
        "link_icon": "search",
        "link": "#"
    }]')),
    ('empty_state', '
It''s possible to use an icon or an image to illustrate the problem.
',
    json('[{
        "component": "empty_state",
        "title": "A critical problem has occurred",
        "icon": "mood-wrrr",
        "description": "Thank you for restarting your browser and crossing your fingers.",
        "link_text": "Close and restart",
        "link_icon": "rotate-clockwise",
        "link": "#"
    }]'));

