-- Alert component
INSERT INTO component(name, icon, description)
VALUES (
        'alert',
        'alert-triangle',
        'A visually distinctive message or notification.'
    );
INSERT INTO parameter(
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
VALUES (
        'alert',
        'title',
        'Title of the alert message.',
        'TEXT',
        FALSE,
        FALSE
    ),
    (
        'alert',
        'icon',
        'Icon name (from tabler-icons.io) to display next to the alert message.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'alert',
        'color',
        'The color theme for the alert message.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'alert',
        'description',
        'Detailed description or content of the alert message.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'alert',
        'link',
        'A URL to link to from the alert message.',
        'URL',
        FALSE,
        TRUE
    ),
    (
        'alert',
        'dismissible',
        'Whether the user can close the alert message.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'alert',
        'important',
        'Set this to TRUE to make the alert message more prominent.',
        'BOOLEAN',
        FALSE,
        TRUE
    ),
    (
        'alert',
        'link_text',
        'Customize the text of the link in the alert message. The default is "Ok".',
        'TEXT',
        FALSE,
        TRUE
    );
-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES (
        'alert',
        'A basic alert message',
        JSON(
            '[{"component":"alert"},{"title":"Attention","description":"This is an important message."}]'
        )
    ),
    (
        'alert',
        'A list of notifications',
        JSON(
            '[{"component":"alert"},
            {"title":"Success","description":"Item successfully added to your cart.", "icon":"check", "color": "green"},
            {"title":"Warning","description":"Your cart is almost full.", "icon":"alert-triangle", "color": "yellow"},
            {"title":"Error","description":"Your cart is full.", "icon":"alert-circle", "color": "red"}
            ]'
        )
    ),
    (
        'alert',
        'A full-featured notification message with a link',
        JSON(
            '[{"component":"alert"},
                {
                    "title": "Your dashboard is ready!",
                    "icon":"analyze",
                    "color":"teal",
                    "dismissible": true,
                    "description":"Your public web dashboard was successfully created.",
                    "link":"/index.sql"
                }]'
        )
    ),
    (
        'alert',
        'An important danger alert message with an icon and color',
        JSON(
            '[{"component":"alert"},
            {
                "title":"Alert",
                "icon":"alert-circle",
                "color":"red",
                "important": true,
                "dismissible": true,
                "description":"SQLPage is entirely free and open source.",
                "link":"https://github.com/lovasoa/SQLPage",
                "link_text":"See source code"
            }]'
        )
    );