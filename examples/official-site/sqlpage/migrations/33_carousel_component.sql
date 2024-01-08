INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'carousel',
        'A carousel is used to display multiple pieces of visual content without taking up too much space.',
        'carousel-horizontal',
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
        'carousel',
        'name',
        'An unique string to identify the caroussel component in the HTML page.',
        'TEXT',
        TRUE,
        FALSE
    ),
    (
        'carousel',
        'title',
        'A name to display at the top of the carousel.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'carousel',
        'indicators',
        'Style of image indicators (square or dot).',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'carousel',
        'vertical',
        'Whether to use the vertical image indicators.',
        'BOOLEAN',
        TRUE,
        TRUE
    ),
    (
        'carousel',
        'controls',
        'Whether to show the control links to go previous or next item.',
        'BOOLEAN',
        TRUE,
        TRUE
    ),
    (
        'carousel',
        'width',
        'Width of the component, between 1 and 12.',
        'NUMBER',
        TRUE,
        FALSE
    ),
    (
        'carousel',
        'center',
        'Whether to center the carousel.',
        'BOOLEAN',
        TRUE,
        TRUE
    ),
    (
        'carousel',
        'fade',
        'Whether to apply the fading effect.',
        'BOOLEAN',
        TRUE,
        TRUE
    ),
    (
        'carousel',
        'image',
        'The URL (absolute or relative) of an image to display in the carousel.',
        'URL',
        FALSE,
        FALSE
    ),
    (
        'carousel',
        'title',
        'Add caption to the slide.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'carousel',
        'description',
        'A short paragraph.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'carousel',
        'description_md',
        'A short paragraph formatted using markdown.',
        'TEXT',
        FALSE,
        TRUE
    );

-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES
    (
        'carousel', 
        'A basic example of carousel', 
            JSON(
                '[
                {"component":"carousel","name":"cats1","title":"Cats","width":6},
                {"image":"https://placekitten.com/408/285"},
                {"image":"https://placekitten.com/408/286"}
                ]'
            )
    ),
    (
        'carousel',
        'An advanced example of carousel with controls',
            JSON(
                '[
                {"component":"carousel","name":"cats2","title":"Cats","width":6,"center":"TRUE","controls":"TRUE"},
                {"image":"https://placekitten.com/408/285","title":"A first cat","description":"The cat (Felis catus), commonly referred to as the domestic cat or house cat, is the only domesticated species in the family Felidae."},
                {"image":"https://placekitten.com/408/286","title":"Another cat"}
                ]'
            )
    );