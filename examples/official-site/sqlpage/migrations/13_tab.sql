INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'tab',
        'Build a tabbed interface, with each tab being a link to a page. Each tab can be in two states: active or inactive.',
        'row-insert-bottom',
        '0.9.5'
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
        'tab',
        'title',
        'Text to display on the tab.',
        'TEXT',
        FALSE,
        FALSE
    ),
    (
        'tab',
        'link',
        'Link to the page to display when the tab is clicked. By default, the link refers to the current page, with a ''tab'' parameter set to the tab''s title and hash set to the id (if passed) - this brings us back to the location of the tab after submission.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'tab',
        'active',
        'Whether the tab is active or not. Defaults to false.',
        'BOOLEAN',
        FALSE,
        TRUE
    ),
    (
        'tab',
        'icon',
        'Name of the icon to display on the tab. See tabler-icons.io for a list of available icons.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'tab',
        'color',
        'Color of the tab. See preview.tabler.io/colors.html for a list of available colors.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'tab',
        'description',
        'Description of the tab. This is displayed when the user hovers over the tab.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'tab',
        'center',
        'Whether the tabs should be centered or not. Defaults to false.',
        'BOOLEAN',
        TRUE,
        TRUE
    )
    ;

INSERT INTO example (component, description, properties)
VALUES (
        'tab',
        'This example shows a very basic set of three tabs. The first tab is active. You could use this at the top of a page for easy navigation.

To implement contents that change based on the active tab, use the `tab` parameter in the page query string.
For example, if the page is `/my-page.sql`, then the first tab will have a link of `/my-page.sql?tab=My+First+tab`.

You could then for instance display contents coming from the database based on the value of the `tab` parameter.
For instance: `SELECT ''text'' AS component, contents_md FROM my_page_contents WHERE tab = $tab`

Note that the example below is completely static, and does not use the `tab` parameter to actually switch between tabs.
View the [dynamic tabs example](examples/tabs.sql).
',
        JSON(
            '[
            { "component": "tab" },
            { "title": "My First tab", "active": true },
            { "title": "This is tab two" },
            { "title": "Third tab is crazy" }
        ]'
        )
    ),
    (
        'tab',
        'This example shows a more sophisticated set of tabs. The tabs are centered, the active tab has a different color, and all the tabs have a custom link and icon.',
        JSON(
            '[
            { "component": "tab", "center": true },
            { "title": "Hero", "link": "?component=hero#component", "icon": "home", "description": "The hero component is a full-width banner with a title and an image." },
            { "title": "Tab", "active": true, "link": "?component=tab#component", "icon": "user", "color": "dark" },
            { "title": "Card", "link": "?component=card#component", "icon": "credit-card" }
        ]'
        )
    )
    ;
