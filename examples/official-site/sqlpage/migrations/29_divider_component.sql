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
    );
-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES      
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
        'A divider with blue text at right', 
            JSON(
                '[
                    {
                        "component":"divider",
                        "contents":"Hello",
                        "position":"right",
                        "color":"blue"
                    }
                ]'
            )
    );