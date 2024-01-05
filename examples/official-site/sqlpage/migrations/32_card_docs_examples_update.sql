INSERT INTO example(component, description, properties) VALUES
    ('card', 'The most basic card', json('[{"component":"card"},{"description":"A"},{"description":"B"},{"description":"C"}]')),
    ('card', 'A card with a Markdown description',
            json('[{"component":"card", "columns": 2}, {"title":"A card with a Markdown description", "description_md": "This is a card with a **Markdown** description. \n\n'||
            'This is useful if you want to display a lot of text in the card, with many options for formatting, such as '||
            '\n - **bold**, \n - *italics*, \n - [links](index.sql), \n - etc."}]')),
    ('card', 'A beautiful card grid with bells and whistles, showing examples of SQLPage features.',
            json('[{"component":"card", "title":"Popular SQLPage features", "columns": 2},
            {"title": "Download as spreadsheet", "link": "?component=csv#component", "description": "Using the CSV component, you can download your data as a spreadsheet.", "icon":"file-plus", "color": "green", "footer_md": "SQLPage can both [read](?component=form#component) and [write](?component=csv#component) **CSV** files."},
            {"title": "Custom components", "link": "/custom_components.sql", "description": "If you know some HTML, you can create your own components for your application.", "icon":"code", "color": "orange", "footer_md": "You can look at the [source of the official components](https://github.com/lovasoa/SQLpage/tree/main/sqlpage/templates) for inspiration."}
    ]')),
    ('card', 'A gallery of images.',
        json('[
            {"component":"card", "title":"My favorite animals in pictures", "columns": 3},
            {"title": "Lynx", "description_md": "The **lynx** is a medium-sized **wild cat** native to Northern, Central and Eastern Europe to Central Asia and Siberia, the Tibetan Plateau and the Himalayas.", "top_image": "https://upload.wikimedia.org/wikipedia/commons/thumb/d/d8/Lynx_lynx-4.JPG/640px-Lynx_lynx-4.JPG", "icon":"star" },
            {"title": "Squirrel", "description_md": "The **chipmunk** is a small, striped rodent of the family Sciuridae. Chipmunks are found in North America, with the exception of the Siberian chipmunk which is found primarily in Asia.", "top_image": "https://upload.wikimedia.org/wikipedia/commons/thumb/b/be/Tamias-rufus-001.jpg/640px-Tamias-rufus-001.jpg" },
            {"title": "Spider", "description_md": "The **jumping spider family** (_Salticidae_) contains more than 600 described genera and about *6000 described species*, making it the largest family of spiders with about 13% of all species.", "top_image": "https://upload.wikimedia.org/wikipedia/commons/thumb/a/ab/Jumping_spiders_%28Salticidae%29.jpg/640px-Jumping_spiders_%28Salticidae%29.jpg" }
        ]')),
    ('card', 'Cards with remote content',
        json('[
            {"component":"card", "title":"Card with embedded remote content", "columns": 2},
            {"title": "Embedded Chart", "embed": "/examples/chart.sql?_sqlpage_embed" },
            {"title": "Description", "description_md": "You can find the sql file that generates the chart [here](https://github.com/lovasoa/SQLpage/tree/main/examples/official-site/examples/chart.sql)" }
        ]'));



