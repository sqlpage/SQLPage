INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'timeline',
        'A list of events with a vertical line connecting them.',
        'git-commit',
        '0.13.0'
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
        'timeline',
        'simple',
        'If set to true, the timeline will be displayed in a condensed format without icons.',
        'BOOLEAN',
        TRUE,
        TRUE
    ),
    (
        'timeline',
        'title',
        'Name of the event.',
        'TEXT',
        FALSE,
        FALSE
    ),
    (
        'timeline',
        'date',
        'Date of the event.',
        'TEXT',
        FALSE,
        FALSE
    ),
    (
        'timeline',
        'icon',
        'Name of the icon to display next to the event. See tabler-icons.io for a list of available icons.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'timeline',
        'color',
        'Color of the icon. See preview.tabler.io/colors.html for a list of available colors.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'timeline',
        'description',
        'Textual description of the event.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'timeline',
        'description_md',
        'Description of the event in Markdown.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'timeline',
        'link',
        'Link to a page with more information about the event.',
        'TEXT',
        FALSE,
        TRUE
    );
INSERT INTO example (component, description, properties)
VALUES (
        'timeline',
        'A basic timeline with just names and dates.',
        JSON(
            '[
            { "component": "timeline", "simple": true },
            { "title": "New message from Elon Musk", "date": "13:00" },
            { "title": "Jeff Bezos assigned task \"work more\" to you.", "date": "yesterday, 16:35" }
        ]'
        )
    ),
    (
        'timeline',
        'A full-fledged timeline with icons, colors, and rich text descriptions.',
        JSON(
            '[
            { "component": "timeline" },
            { "title": "v0.13.0 was just released !", "link": "https://github.com/lovasoa/SQLpage/releases/", "date": "2023-10-16", "icon": "brand-github", "color": "green", "description_md": "This version introduces the `timeline` component." },
            { "title": "They are talking about us...", "description_md": "[This article](https://www.postgresql.org/about/news/announcing-sqlpage-build-dynamic-web-applications-in-sql-2672/) on the official PostgreSQL website mentions SQLPage.", "date": "2023-07-12", "icon": "database", "color": "blue" }
        ]'
        )
    )
    ;