INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'divider',
        'Dividers help organize content and make the interface layout clear and uncluttered.',
        'separator',
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
        'divider',
        'contents',
        'A text in the divider.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'divider',
        'position',
        'Position of the text (e.g. left, right).',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'divider',
        'color',
        'The name of a color for this span of text.',
        'COLOR',
        TRUE,
        TRUE
    ),
    (
        'divider',
        'size',
        'The size of the divider text, from 1 to 6.',
        'INTEGER',
        TRUE,
        TRUE
    ),
    (
        'divider',
        'bold',
        'Whether the text is bold.',
        'BOOLEAN',
        TRUE,
        TRUE
    ),
    (
        'divider',
        'italics',
        'Whether the text is italicized.',
        'BOOLEAN',
        TRUE,
        TRUE
    ),
    (
        'divider',
        'underline',
        'Whether the text is underlined.',
        'BOOLEAN',
        TRUE,
        TRUE
    ),
    (
        'divider',
        'link',
        'URL of the link for the divider text. Available only when contents is present.',
        'URL',
        TRUE,
        TRUE
    );

-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES
    (
        'divider', 
        'An empty divider', 
            JSON(
                '[
                    {
                        "component":"divider"
                    }
                ]'
            )
    ),
    (
        'divider', 
        'A divider with centered text', 
            JSON(
                '[
                    {
                        "component":"divider",
                        "contents":"Hello"
                    }
                ]'
            )
    ),
    (
        'divider', 
        'A divider with text at left', 
            JSON(
                '[
                    {
                        "component":"divider",
                        "contents":"Hello",
                        "position":"left"
                    }
                ]'
            )
    ),
    (
        'divider', 
        'A divider with blue text and a link', 
            JSON(
                '[
                    {
                        "component":"divider",
                        "contents":"SQLPage components",
                        "link":"/documentation.sql",
                        "color":"blue"
                    }
                ]'
            )
    ),
    (
        'divider', 
        'A divider with bold, italic, and underlined text', 
            JSON(
                '[
                    {
                        "component":"divider",
                        "contents":"Important notice",
                        "position":"left",
                        "color":"red",
                        "size":5,
                        "bold":true,
                        "italics":true,
                        "underline":true
                    }
                ]'
            )
    );