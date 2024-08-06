DELETE FROM component WHERE name = 'card';

INSERT INTO component(name, icon, description) VALUES
    ('card', 'credit-card', 'A grid where each element is a small card that displays a piece of data.');
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'card', * FROM (VALUES
    -- top level
    ('title', 'Text header at the top of the list of cards.', 'TEXT', TRUE, TRUE),
    ('description', 'A short paragraph displayed below the title.', 'TEXT', TRUE, TRUE),
    ('description_md', 'A short paragraph displayed below the title - formatted using markdown.', 'TEXT', TRUE, TRUE),
    ('columns', 'The number of columns in the grid of cards. This is just a hint, the grid will adjust dynamically to the user''s screen size, rendering fewer columns if needed to fit the contents. To control the size of cards individually, use the `width` row-level property instead.', 'INTEGER', TRUE, TRUE),
    -- item level
    ('title', 'Name of the card, displayed at the top.', 'TEXT', FALSE, FALSE),
    ('description', 'The body of the card, where you put the main text contents of the card.
        This does not support rich text formatting, only plain text.
        If you want to use rich text formatting, use the `description_md` property instead.', 'TEXT', FALSE, TRUE),
    ('description_md', '
        The body of the card, in Markdown format.
        This is useful if you want to display a lot of text in the card, with many options for formatting, such as
        line breaks, **bold**, *italics*, lists, #titles, [links](target.sql), ![images](photo.jpg), etc.', 'TEXT', FALSE, TRUE),
    ('top_image', 'The URL (absolute or relative) of an image to display at the top of the card.', 'URL', FALSE, TRUE),
    ('footer', 'Muted text to display at the bottom of the card.', 'TEXT', FALSE, TRUE),
    ('footer_md', 'Muted text to display at the bottom of the card, with rich text formatting in Markdown format.', 'TEXT', FALSE, TRUE),
    ('link', 'An URL to which the user should be taken when they click on the card.', 'URL', FALSE, TRUE),
    ('footer_link', 'An URL to which the user should be taken when they click on the footer.', 'URL', FALSE, TRUE),
    ('style', 'Inline style property to your iframe embed code. For example "background-color: #FFFFFF"', 'TEXT', FALSE, TRUE),
    ('icon', 'Name of an icon to display on the left side of the card.', 'ICON', FALSE, TRUE),
    ('color', 'The name of a color, to be displayed on the left of the card to highlight it.', 'COLOR', FALSE, TRUE),
    ('background_color', 'The background color of the card.', 'COLOR', FALSE, TRUE),
    ('active', 'Whether this item in the grid is considered "active". Active items are displayed more prominently.', 'BOOLEAN', FALSE, TRUE),
    ('width', 'The width of the card, between 1 (smallest) and 12 (full-width). The default width is 3, resulting in 4 cards per line.', 'INTEGER', FALSE, TRUE)
) x;
INSERT INTO parameter(component, name, description_md, type, top_level, optional) SELECT 'card', * FROM (VALUES
    ('embed', 'A url whose contents will be fetched and injected into the body of this card.
        This can be used to inject arbitrary html content, but is especially useful for injecting
        the output of other sql files rendered by SQLPage. For the latter case you can pass the
        `?_sqlpage_embed` query parameter, which will skip the shell layout', 'TEXT', FALSE, TRUE),
    ('embed_mode', 'Set to ''iframe'' to embed the target (specified through embed property) in an iframe.
        Unless this is explicitly set, the embed target is fetched and injected within the parent page. If embed_mode is set to iframe,
        You can also set height and width parameters to configure the appearance and the sandbox and allow parameters to configure
        security aspects of the iframe. Refer to the [MDN page](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe)
        for an explanation of these parameters.', 'TEXT', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('card', 'A beautiful card grid with bells and whistles, showing examples of SQLPage features.',
            json('[{"component":"card", "title":"Popular SQLPage features", "columns": 2},
            {"title": "Download as spreadsheet", "link": "?component=csv#component", "description": "Using the CSV component, you can download your data as a spreadsheet.", "icon":"file-plus", "color": "green", "footer_md": "SQLPage can both [read](?component=form#component) and [write](?component=csv#component) **CSV** files."},
            {"title": "Custom components", "link": "/custom_components.sql", "description": "If you know some HTML, you can create your own components for your application.", "icon":"code", "color": "orange", "footer_md": "You can look at the [source of the official components](https://github.com/lovasoa/SQLpage/tree/main/sqlpage/templates) for inspiration."}
    ]')),
    ('card', 'You can use cards to display a dashboard with quick access to important information. Use [markdown](https://www.markdownguide.org/basic-syntax) to format the text.',
        json('[
            {"component": "card", "columns": 4},
            {"description_md": "**152** sales today", "active": true, "icon": "currency-euro"},
            {"description_md": "**13** new users", "icon": "user-plus", "color": "green"},
            {"description_md": "**2** complaints", "icon": "alert-circle", "color": "danger", "link": "?view_complaints", "background_color": "red-lt"},
            {"description_md": "**1** pending support request", "icon": "mail-question", "color": "warning"}
        ]')),
    ('card', 'A gallery of images.',
        json('[
            {"component":"card", "title":"My favorite animals in pictures", "columns": 3},
            {"title": "Lynx", "description_md": "The **lynx** is a medium-sized **wild cat** native to Northern, Central and Eastern Europe to Central Asia and Siberia, the Tibetan Plateau and the Himalayas.", "top_image": "https://upload.wikimedia.org/wikipedia/commons/thumb/d/d8/Lynx_lynx-4.JPG/640px-Lynx_lynx-4.JPG", "icon":"star" },
            {"title": "Squirrel", "description_md": "The **chipmunk** is a small, striped rodent of the family Sciuridae. Chipmunks are found in North America, with the exception of the Siberian chipmunk which is found primarily in Asia.", "top_image": "https://upload.wikimedia.org/wikipedia/commons/thumb/b/be/Tamias-rufus-001.jpg/640px-Tamias-rufus-001.jpg" },
            {"title": "Spider", "description_md": "The **jumping spider family** (_Salticidae_) contains more than 600 described genera and about *6000 described species*, making it the largest family of spiders with about 13% of all species.", "top_image": "https://upload.wikimedia.org/wikipedia/commons/thumb/a/ab/Jumping_spiders_%28Salticidae%29.jpg/640px-Jumping_spiders_%28Salticidae%29.jpg" }
        ]')),
    ('card', 'Beautifully colored cards with variable width. The blue card (width 6) takes half the screen, whereas of the red and green cards have the default width of 3',
        json('[
            {"component":"card", "title":"Beautifully colored cards" },
            {"title": "Red card", "color": "red", "background_color": "red-lt", "description": "Penalty! You are out!", "icon":"play-football" },
            {"title": "Blue card", "color": "blue", "width": 6, "background_color": "blue-lt", "description": "The Blue Card facilitates migration of foreigners to Europe.", "icon":"currency-euro" },
            {"title": "Green card", "color": "green", "background_color": "green-lt", "description": "Welcome to the United States of America !", "icon":"user-dollar" }
        ]')),
    ('card', 'Cards with remote content',
        json('[
            {"component":"card", "title":"Card with embedded remote content", "columns": 2},
            {"title": "Embedded Chart", "embed": "/examples/chart.sql?_sqlpage_embed", "footer_md": "You can find the sql file that generates the chart [here](https://github.com/lovasoa/SQLpage/tree/main/examples/official-site/examples/chart.sql)"  },
            {"title": "Embedded Video", "embed": "https://www.youtube.com/embed/mXdgmSdaXkg", "allow": "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share", "embed_mode": "iframe", "height": "350" }
        ]'));
