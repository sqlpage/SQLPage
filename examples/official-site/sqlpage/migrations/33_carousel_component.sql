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
        'Name of the carousel.',
        'TEXT',
        TRUE,
        FALSE
    ),
    (
        'carousel',
        'title',
        'Title of the carousel.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'carousel',
        'indicators',
        'Style of image indicators (e.g. square, dot).',
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
        'contents',
        'Add paragraph to the slide.',
        'TEXT',
        FALSE,
        TRUE
    );


select
    'carousel' as component,
    'somecats' as name,
    'Some cats' as title,
    'square' as indicators,  --square or dot
    FALSE as vertical,
    TRUE as controls,
    6 as width,
    TRUE as center,
    FALSE as fade;

select
    "https://placekitten.com/408/285" as image,
    'Second cat' as title,
    'Some representative placeholder content for the second slide.' as contents;
select
    "https://placekitten.com/408/286" as image,
    'Third cat' as title;

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
                {"image":"https://placekitten.com/408/285","title":"A first cat","contents":"The cat (Felis catus), commonly referred to as the domestic cat or house cat, is the only domesticated species in the family Felidae."},
                {"image":"https://placekitten.com/408/286","title":"Another cat"}
                ]'
            )
    );