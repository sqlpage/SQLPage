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
        TRUE,
        FALSE
    ),
    (
        'alert',
        'icon',
        'Icon name (from tabler-icons.io) to display next to the alert message.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'alert',
        'color',
        'The color theme for the alert message.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'alert',
        'description',
        'Detailed description or content of the alert message.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'alert',
        'description_md',
        'Detailed description or content of the alert message, in Markdown format, allowing you to use rich text formatting, including **bold** and *italic* text.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'alert',
        'dismissible',
        'Whether the user can close the alert message.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'alert',
        'important',
        'Set this to TRUE to make the alert message more prominent.',
        'BOOLEAN',
        TRUE,
        TRUE
    ),
    (
        'alert',
        'link',
        'A URL to link to from the alert message.',
        'URL',
        TRUE,
        TRUE
    ),
    (
        'alert',
        'link_text',
        'Customize the text of the link in the alert message. The default is "Ok".',
        'TEXT',
        TRUE,
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
        'title',
        'Customize the text of the link in the alert message. The default is "Ok".',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'alert',
        'color',
        'Customize the color of the link.',
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
            '[{"component":"alert", "title":"Attention", "description":"This is an important message."}]'
        )
    ),
    (
        'alert',
        'A list of notifications',
        JSON(
            '[
            {"component" :"alert", "title":"Success","description":"Item successfully added to your cart.", "icon":"check", "color": "green"},
            {"component":"alert", "title":"Warning","description":"Your cart is almost full.", "icon":"alert-triangle", "color": "yellow"},
            {"component":"alert", "title":"Error","description":"Your cart is full.", "icon":"alert-circle", "color": "red"}
            ]'
        )
    ),
    (
        'alert',
        'A full-featured notification message with multiple links',
        JSON(
            '[
                {
                    "component" :"alert",
                    "title": "Your dashboard is ready!",
                    "icon":"analyze",
                    "color":"teal",
                    "dismissible": true,
                    "description":"Your public web dashboard was successfully created."
                },
                {
                    "link":"dashboard.sql",
                    "title": "View your dashboard"
                },
                { 
                    "link" :"index.sql",
                    "title": "Back to home",
                    "color": "secondary"
                }
            ]'
        )
    ),
    (
        'alert',
        'An important danger alert message with an icon and color',
        JSON(
            '[
            {
                "component":"alert",
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
    ),
    (
        'alert',
        'An alert message with a Markdown-formatted description',
        JSON(
            '[
        {
            "component":"alert",
            "title":"Free and open source",
            "icon": "free-rights",
            "color": "info",
            "description_md":"*SQLPage* is entirely free and open source. You can **contribute** to it on [GitHub](https://github.com/lovasoa/SQLPage)."
        }]'
        )
    );