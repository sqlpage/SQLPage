INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('empty_state', 'info-circle', 'Displays a large placeholder message to communicate a single information to the user and invite them to take action.

Typically includes a title, an optional icon/image, descriptive text (rich text formatting and images supported via Markdown), and a call-to-action button.

Ideal for first-use screens, empty data sets, "no results" pages, or error messages.', '0.35.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'empty_state', * FROM (VALUES
    ('title','Description of the empty state.','TEXT',TRUE,FALSE),
    ('header','Text displayed on the top of the empty state.','TEXT',TRUE,TRUE),
    ('icon','Name of an icon to be displayed on the top of the empty state.','ICON',TRUE,TRUE),
    ('image','The URL (absolute or relative) of an image to display at the top of the empty state.','URL',TRUE,TRUE),
    ('description','A short text displayed below the title.','TEXT',TRUE,TRUE),
    ('link_text','The text displayed on the button.','TEXT',TRUE,FALSE),
    ('link_icon','Name of an icon to be displayed on the left side of the button.','ICON',TRUE,FALSE),
    ('link','The URL to which the button should navigate when clicked.','URL',TRUE,FALSE),
    ('class','Class attribute added to the container in HTML. It can be used to apply custom styling to this item through css.','TEXT',TRUE,TRUE),
    ('id','ID attribute added to the container in HTML. It can be used to target this item through css or for scrolling to this item through links (use "#id" in link url).','TEXT',TRUE,TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('empty_state', '
This example shows how to create a 404-style "Not Found" empty state with 
 - a prominent header displaying "404",
 - a helpful description suggesting to adjust search parameters, and
 - a "Search again" button with a search icon that links back to the search page.
',
    json('[{
        "component": "empty_state",
        "title": "No results found",
        "header": "404",
        "description": "Try adjusting your search or filter to find what you''re looking for.",
        "link_text": "Search again",
        "link_icon": "search",
        "link": "#not-found",
        "id": "not-found"
    }]')),
    ('empty_state', '
It''s possible to use an icon or an image to illustrate the problem.
',
    json('[{
        "component": "empty_state",
        "title": "A critical problem has occurred",
        "icon": "mood-wrrr",
        "description_md": "SQLPage can do a lot of things, but this is not one of them.

Please restart your browser and **cross your fingers**.",
        "link_text": "Close and restart",
        "link_icon": "rotate-clockwise",
        "link": "#"
    }]'));

